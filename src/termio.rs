use std::{io::ErrorKind, mem::MaybeUninit, os::fd::RawFd, sync::atomic::{AtomicU32, Ordering}};

use crate::{epoll::{EPoll, Event, Events}, escape::{ESCAPE, InputSequence}};

// if konsole would support this, that would be so much nicer: https://gist.github.com/rockorager/e695fb2924d36b2bcf1fff4a3704bd83
static SIGWINCH_NR: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub struct TermIO {
    wfd: RawFd,
    rfd: RawFd,
    buffer: Box<[u8]>,
    buffer_size: usize,
    buffer_index: usize,
    events: Box<[Event]>,
    orig_termios: libc::termios,
    sigwinch_nr: u32,
    epoll: EPoll,
}

const READ_SIZE: usize = 1024;
const BUFFER_SIZE: usize = READ_SIZE + 4; // room for unget
const EPOLL_BUFFER_SIZE: usize = 16;

extern "C" fn handle_sigwinch(_: libc::c_int) {
    SIGWINCH_NR.fetch_add(1, Ordering::AcqRel);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowSize {
    pub rows: u32,
    pub columns: u32,
}

impl TermIO {
    #[inline]
    pub fn from_stdio() -> std::io::Result<Self> {
        Self::new(libc::STDOUT_FILENO, libc::STDERR_FILENO)
    }

    pub fn from_tty() -> std::io::Result<Self> {
        let fd = unsafe { libc::open(b"/dev/tty\0".as_ptr() as *const i8, libc::O_CLOEXEC | libc::O_RDWR) };

        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Self::new(fd, fd)
    }

    #[inline]
    pub fn from_fallback() -> std::io::Result<Self> {
        Self::from_tty().or_else(|_| Self::from_stdio())
    }

    pub fn new(wfd: RawFd, rfd: RawFd) -> std::io::Result<Self> {
        let epoll = EPoll::new()?;
        let mut orig_termios = MaybeUninit::<libc::termios>::zeroed();

        let res = unsafe { libc::tcgetattr(rfd, orig_termios.as_mut_ptr()) };
        if res == -1 {
            return Err(std::io::Error::last_os_error());
        }

        let orig_termios = unsafe { orig_termios.assume_init_mut() };
        let mut new_termios = orig_termios.clone();

        // turn off canonical mode
        new_termios.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
        new_termios.c_oflag |= libc::ONLCR;
        new_termios.c_cflag |= libc::CS8;
        new_termios.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);

        // minimum of number input read.
        new_termios.c_cc[libc::VMIN] = 0;
        new_termios.c_cc[libc::VTIME] = 0;

        let res = unsafe { libc::tcsetattr(rfd, libc::TCSANOW, &new_termios) };
        if res == -1 {
            return Err(std::io::Error::last_os_error());
        }

        let mut app = Self {
            wfd, rfd,
            buffer: vec![0u8; BUFFER_SIZE].into_boxed_slice(),
            buffer_size: 0,
            buffer_index: 0,
            orig_termios: *orig_termios,
            sigwinch_nr: 0,
            epoll,
            events: vec![Event::default(); EPOLL_BUFFER_SIZE].into_boxed_slice(),
        };

        app.epoll.add(rfd, Events::In | Events::ReadHangup, 0)?;

        // CSI ? 1049 h   Enable alternative screen buffer
        // CSI ?   25 l   Hide cursor (DECTCEM), VT220
        // CSI ?    7 l   No Auto-Wrap Mode (DECAWM), VT100.
        // CSI 2 J        Clear entire screen
        //app.write(b"\x1B[?1049h\x1B[?25l\x1B[?7l\x1B[2J")?;

        if SIGWINCH_NR.fetch_add(1, Ordering::AcqRel) == 0 {
            let handler = handle_sigwinch as extern "C" fn(libc::c_int);
            let handler = handler as *const extern "C" fn(libc::c_int);
            let res = unsafe { libc::signal(libc::SIGWINCH, handler as libc::sighandler_t) };

            if res == libc::SIG_ERR {
                return Err(std::io::Error::last_os_error());
            }
        }

        Ok(app)
    }

    #[inline]
    pub fn rfd(&self) -> RawFd {
        self.rfd
    }

    #[inline]
    pub fn wfd(&self) -> RawFd {
        self.wfd
    }

    #[inline]
    pub fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        self.write(s.as_bytes())
    }

    pub fn write(&mut self, mut bytes: &[u8]) -> std::io::Result<()> {
        while !bytes.is_empty() {
            let res = unsafe { libc::write(self.wfd, bytes.as_ptr() as * const libc::c_void, bytes.len() )};

            if res < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(err);
            }

            bytes = &bytes[res as usize..];
        }

        Ok(())
    }

    pub fn read_byte(&mut self) -> std::io::Result<Option<u8>> {
        if self.buffer_index < self.buffer_size {
            let byte = self.buffer[self.buffer_index];
            self.buffer_index += 1;
            return Ok(Some(byte));
        }

        let res = loop {
            let res = unsafe { libc::read(self.rfd, self.buffer.as_mut_ptr() as *mut libc::c_void, READ_SIZE) };

            if res < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(err);
            }

            if res == 0 {
                return Ok(None);
            }

            break res;
        };

        let byte = self.buffer[0];
        self.buffer_index = 1;
        self.buffer_size = res as usize;

        Ok(Some(byte))
    }

    pub fn peek_byte(&mut self) -> std::io::Result<Option<u8>> {
        if self.buffer_index < self.buffer_size {
            let byte = self.buffer[self.buffer_index];
            return Ok(Some(byte));
        }

        let res = unsafe { libc::read(self.rfd, self.buffer.as_mut_ptr() as *mut libc::c_void, self.buffer.len()) };

        if res < 0 {
            return Err(std::io::Error::last_os_error());
        }

        if res == 0 {
            return Ok(None);
        }

        let byte = self.buffer[0];
        self.buffer_index = 0;
        self.buffer_size = res as usize;

        Ok(Some(byte))
    }

    fn unget_byte(&mut self, byte: u8) {
        if self.buffer_index == 0 {
            self.buffer.copy_within(0..self.buffer_size, 1);
            self.buffer_size += 1;
            self.buffer[0] = byte;
        } else {
            self.buffer_index -= 1;
            self.buffer[self.buffer_index] = byte;
        }
    }

    fn unget_slice(&mut self, bytes: &[u8]) {
        if self.buffer_index < bytes.len() {
            self.buffer.copy_within(0..self.buffer_size, bytes.len());
            self.buffer_size += bytes.len();
            self.buffer[0..bytes.len()].copy_from_slice(bytes);
        } else {
            self.buffer_index -= bytes.len();
            self.buffer[self.buffer_index..self.buffer_index + bytes.len()].copy_from_slice(bytes);
        }
    }

    pub fn window_size(&mut self) -> std::io::Result<WindowSize> {
        let mut winsize = libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let res = unsafe { libc::ioctl(self.rfd, libc::TIOCGWINSZ, (&mut winsize) as *mut libc::winsize) };
        if res < 0 {
            return Err(std::io::Error::last_os_error());
        }

        return Ok(WindowSize {
            rows: winsize.ws_row.into(),
            columns: winsize.ws_col.into(),
        });
    }

    pub fn wait(&mut self) -> std::io::Result<Option<InputSequence>> {
        let sigwinch_nr = SIGWINCH_NR.load(Ordering::Relaxed);
        if self.sigwinch_nr != sigwinch_nr {
            self.sigwinch_nr = sigwinch_nr;

            let winsize = self.window_size()?;
            return Ok(Some(InputSequence::WindowSize {
                rows: winsize.rows,
                columns: winsize.columns,
            }));
        }

        loop {
            match self.epoll.wait(&mut self.events, None) {
                Ok(count) => {
                    let mut has_data = false;

                    for event in &self.events[..count] {
                        if event.events().contains(Events::ReadHangup) {
                            return Ok(None);
                        }

                        if event.events().contains(Events::In) {
                            has_data = true;
                        }
                    }

                    if has_data {
                        return self.read();
                    }
                }
                Err(err) => {
                    if err.kind() == ErrorKind::Interrupted {
                        let sigwinch_nr = SIGWINCH_NR.load(Ordering::Relaxed);
                        if self.sigwinch_nr != sigwinch_nr {
                            self.sigwinch_nr = sigwinch_nr;

                            let winsize = self.window_size()?;
                            return Ok(Some(InputSequence::WindowSize {
                                rows: winsize.rows,
                                columns: winsize.columns,
                            }));
                        }
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }

    pub fn read(&mut self) -> std::io::Result<Option<InputSequence>> {
        loop {
            let sigwinch_nr = SIGWINCH_NR.load(Ordering::Relaxed);
            if self.sigwinch_nr != sigwinch_nr {
                self.sigwinch_nr = sigwinch_nr;

                let winsize = self.window_size()?;
                return Ok(Some(InputSequence::WindowSize {
                    rows: winsize.rows,
                    columns: winsize.columns,
                }));
            }

            let Some(byte) = self.read_byte()? else {
                return Ok(None);
            };

            if byte != ESCAPE {
                // parse UTF-8
                if byte >= 0xC0 {
                    let mut codepoint: u32 = byte.into();
                    // UTF-8 multi-byte sequence

                    if byte >= 0xF0 {
                        // 4 bytes
                        codepoint &= 0x07;

                        let b2 = self.peek_byte()?;

                        if let Some(b2) = b2 && is_cont(b2) {
                            self.buffer_index += 1;

                            let b3 = self.peek_byte()?;

                            if let Some(b3) = b3 && is_cont(b3) {
                                self.buffer_index += 1;

                                codepoint <<= 6;
                                codepoint |= b3 as u32 & 0x3F;

                                let b4 = self.peek_byte()?;

                                if let Some(b4) = b4 && is_cont(b4) {
                                    self.buffer_index += 1;

                                    codepoint <<= 6;
                                    codepoint |= b4 as u32 & 0x3F;

                                    return Ok(Some(InputSequence::Char(unsafe { char::from_u32_unchecked(codepoint) })));
                                } else {
                                    self.unget_slice(&[b2, b3]);
                                    return Ok(Some(InputSequence::Char(surrogate_escape(byte))));
                                }
                            } else {
                                self.unget_byte(b2);
                                return Ok(Some(InputSequence::Char(surrogate_escape(byte))));
                            }
                        } else {
                            return Ok(Some(InputSequence::Char(surrogate_escape(byte))));
                        }
                    } else if byte >= 0xE0 {
                        // 3 bytes
                        codepoint &= 0x0F;

                        let b2 = self.peek_byte()?;

                        if let Some(b2) = b2 && is_cont(b2) {
                            self.buffer_index += 1;

                            let b3 = self.peek_byte()?;

                            if let Some(b3) = b3 && is_cont(b3) {
                                self.buffer_index += 1;

                                codepoint <<= 6;
                                codepoint |= b3 as u32 & 0x3F;

                                return Ok(Some(InputSequence::Char(unsafe { char::from_u32_unchecked(codepoint) })));
                            } else {
                                self.unget_byte(b2);
                                return Ok(Some(InputSequence::Char(surrogate_escape(byte))));
                            }
                        } else {
                            return Ok(Some(InputSequence::Char(surrogate_escape(byte))));
                        }
                    } else {
                        // 2 bytes
                        codepoint &= 0x1F;

                        let b2 = self.peek_byte()?;

                        if let Some(b2) = b2 && is_cont(b2) {
                            self.buffer_index += 1;

                            codepoint <<= 6;
                            codepoint |= b2 as u32 & 0x3F;

                            return Ok(Some(InputSequence::Char(unsafe { char::from_u32_unchecked(codepoint) })));
                        } else {
                            return Ok(Some(InputSequence::Char(surrogate_escape(byte))));
                        }
                    }
                }

                return Ok(Some(InputSequence::Char(byte.into())));
            }

            // else ESC
            // https://en.wikipedia.org/wiki/ANSI_escape_code#Terminal_input_sequences

            let Some(byte) = self.peek_byte()? else {
                return Ok(Some(InputSequence::Char(ESCAPE.into())));
            };

            if byte == b'[' {
                // TODO: more generic list of number parsing
                self.buffer_index += 1;

                let Some(mut byte) = self.peek_byte()? else {
                    // ignore unsupported escape sequence
                    continue;
                };

                let private = byte == b'?';
                if private {
                    self.buffer_index += 1;

                    let Some(next) = self.peek_byte()? else {
                        // ignore unsupported escape sequence
                        continue;
                    };

                    byte = next;
                }

                let mut modifier: u32 = 1;

                if byte.is_ascii_digit() {
                    modifier = 0;
                    while byte.is_ascii_digit() {
                        // check for integer overflow?
                        modifier *= 10;
                        modifier += (byte - b'0') as u32;

                        self.buffer_index += 1;
                        let Some(next) = self.peek_byte()? else {
                            break;
                        };

                        byte = next;
                    }
                }

                if byte == b';' {
                    self.buffer_index += 1;

                    let key_code = modifier;
                    modifier = 1;

                    let Some(next) = self.peek_byte()? else {
                        // ignore unsupported escape sequence
                        continue;
                    };

                    byte = next;

                    if byte.is_ascii_digit() {
                        modifier = 0;
                        while byte.is_ascii_digit() {
                            // check for integer overflow?
                            modifier *= 10;
                            modifier += (byte - b'0') as u32;

                            self.buffer_index += 1;
                            let Some(next) = self.peek_byte()? else {
                                break;
                            };

                            byte = next;
                        }
                    }

                    if byte != b'~' {
                        // ignore unsupported escape sequence
                        continue;
                    }

                    self.buffer_index += 1;

                    if private {
                        if byte == b'R' {
                            return Ok(Some(InputSequence::CursorPos { row: key_code, column: modifier }))
                        }

                        // ignore unsupported escape sequence
                        continue;
                    }

                    return Ok(Some(InputSequence::KeyCode { key_code, modifier }));

                }

                if byte == ESCAPE {
                    // ignore unsupported escape sequence
                    continue;
                }

                self.buffer_index += 1;

                if private {
                    // ignore unsupported escape sequence
                    continue;
                }

                return Ok(Some(InputSequence::Modifier { modifier, char: byte.into() }));
            }

            if byte == b'O' {
                // SS3
                self.buffer_index += 1;

                let Some(next) = self.read_byte()? else {
                    return Ok(Some(InputSequence::Alt(byte.into())));
                };

                return Ok(Some(InputSequence::SingleShift3(next.into())));
            }

            self.buffer_index += 1;
            return Ok(Some(InputSequence::Alt(byte.into())));
        }
    }
}

#[inline]
fn is_cont(byte: u8) -> bool {
    byte >= 0x80 && byte < 0xC0
}

#[inline]
fn surrogate_escape(byte: u8) -> char {
    unsafe { char::from_u32_unchecked(0xDC00 + byte as u32) }
}

impl std::fmt::Write for TermIO {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.write_str(s).or(Err(std::fmt::Error))
    }
}

impl Drop for TermIO {
    fn drop(&mut self) {
        let _ = unsafe { libc::tcsetattr(self.rfd, libc::TCSANOW, &self.orig_termios) };

        // CSI 0 m        Reset or normal, all attributes become turned off
        // CSI ?   25 h   Show cursor (DECTCEM), VT220
        // CSI ?    7 h   Auto-Wrap Mode (DECAWM), VT100
        // CSI ? 1049 l   Disable alternative screen buffer
        //let _ = self.write(b"\x1B[0m\x1B[?25h\x1B[?7h\x1B[?1049l");

        if self.wfd != libc::STDOUT_FILENO && self.wfd != libc::STDERR_FILENO {
            let _ = unsafe { libc::close(self.wfd) };
        }

        if self.rfd != libc::STDIN_FILENO && self.wfd != self.rfd {
            let _ = unsafe { libc::close(self.rfd) };
        }
    }
}
