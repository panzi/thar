use std::{io::{BufWriter, ErrorKind, Write}, mem::MaybeUninit, os::fd::RawFd, sync::atomic::{AtomicU32, Ordering}};

use crate::{borrowed_fd::BorrowedFd, char_width::crop, color::{Color, Color16, Rgb}, epoll::{EPoll, Events}, event::{ESCAPE, ESCAPE_EVENT, Event, Key}, rich_text::{RichText, RichTextCode}, style::{FontStyle, FontWeight, TextDecoration, TextStyle}};

// if konsole would support this, that would be so much nicer: https://gist.github.com/rockorager/e695fb2924d36b2bcf1fff4a3704bd83
static SIGWINCH_NR: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub struct TermIO {
    epoll: EPoll,
    orig_termios: libc::termios,
    window_size: WindowSize,
    writer: BufWriter<BorrowedFd>,
    uint_buffer: Vec<u32>,
    wfd: RawFd,
    rfd: RawFd,
    buffer: Box<[u8]>,
    buffer_size: usize,
    buffer_index: usize,
    events: Box<[crate::epoll::Event]>,
    sigwinch_nr: u32,
    mouse_enabled: bool,
    inverted: bool,
    default_fg: Color,
    default_bg: Color,
}

const READ_SIZE: usize = 1024;
const BUFFER_SIZE: usize = READ_SIZE + 4; // room for unget
const EPOLL_BUFFER_SIZE: usize = 16;

extern "C" fn handle_sigwinch(_: libc::c_int) {
    SIGWINCH_NR.fetch_add(1, Ordering::AcqRel);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WindowSize {
    pub rows: u32,
    pub columns: u32,
}

pub const FG_DEFAULT: &[u8] = b"\x1B[39m";
pub const BG_DEFAULT: &[u8] = b"\x1B[49m";

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
            wfd,
            rfd,
            writer: BufWriter::new(BorrowedFd::new(wfd)),
            buffer: vec![0u8; BUFFER_SIZE].into_boxed_slice(),
            buffer_size: 0,
            buffer_index: 0,
            orig_termios: *orig_termios,
            sigwinch_nr: 0,
            epoll,
            uint_buffer: Vec::with_capacity(8),
            events: vec![crate::epoll::Event::default(); EPOLL_BUFFER_SIZE].into_boxed_slice(),
            mouse_enabled: false,
            window_size: WindowSize::default(),
            inverted: false,
            default_fg: Color::Default,
            default_bg: Color::Default,
        };

        app.epoll.add(rfd, Events::In | Events::ReadHangup, 0)?;

        // CSI ? 1049 h   Enable alternative screen buffer
        // CSI ?   25 l   Hide cursor (DECTCEM), VT220
        // CSI ?    7 l   No Auto-Wrap Mode (DECAWM), VT100.
        // CSI 2 J        Clear entire screen
        app.write(b"\x1B[?1049h\x1B[?25l\x1B[?7l\x1B[2J")?;
        //app.write(b"\x1B[?25l\x1B[?7l\x1B[2J")?;
        app.flush()?;
        app.refresh_window_size()?;

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
        self.writer.write_all(s.as_bytes())
    }

    #[inline]
    pub fn write(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(bytes)
    }

    #[inline]
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    #[inline]
    pub fn default_fg(&self) -> Color {
        self.default_fg
    }

    #[inline]
    pub fn default_bg(&self) -> Color {
        self.default_bg
    }

    #[inline]
    pub fn set_default_fg(&mut self, color: Color) {
        self.default_fg = color;
    }

    #[inline]
    pub fn set_default_bg(&mut self, color: Color) {
        self.default_bg = color;
    }

    #[inline]
    pub fn raw_fg_default(&mut self) -> std::io::Result<()> {
        match self.default_fg {
            Color::Default => self.writer.write_all(FG_DEFAULT),
            Color::Color16(color) => self.raw_fg16(color),
            Color::Rgb { r, g, b } => self.raw_fg_rgb(Rgb { r, g, b }),
        }
    }

    #[inline]
    pub fn raw_bg_default(&mut self) -> std::io::Result<()> {
        match self.default_bg {
            Color::Default => self.writer.write_all(BG_DEFAULT),
            Color::Color16(color) => self.raw_bg16(color),
            Color::Rgb { r, g, b } => self.raw_bg_rgb(Rgb { r, g, b }),
        }
    }

    #[inline]
    pub fn raw_fg_rgb(&mut self, Rgb { r, g, b }: Rgb) -> std::io::Result<()> {
        write!(self.writer, "\x1B[38;2;{r};{g};{b}m")
    }

    #[inline]
    pub fn raw_bg_rgb(&mut self, Rgb { r, g, b }: Rgb) -> std::io::Result<()> {
        write!(self.writer, "\x1B[48;2;{r};{g};{b}m")
    }

    #[inline]
    pub fn raw_fg16(&mut self, color: Color16) -> std::io::Result<()> {
        self.writer.write_all(color.fg())
    }

    #[inline]
    pub fn raw_bg16(&mut self, color: Color16) -> std::io::Result<()> {
        self.writer.write_all(color.bg())
    }

    #[inline]
    pub fn fg_default(&mut self) -> std::io::Result<()> {
        if self.inverted {
            self.raw_bg_default()
        } else {
            self.raw_fg_default()
        }
    }

    #[inline]
    pub fn bg_default(&mut self) -> std::io::Result<()> {
        if self.inverted {
            self.raw_fg_default()
        } else {
            self.raw_bg_default()
        }
    }

    #[inline]
    pub fn fg_rgb(&mut self, color: Rgb) -> std::io::Result<()> {
        if self.inverted {
            self.raw_bg_rgb(color)
        } else {
            self.raw_fg_rgb(color)
        }
    }

    #[inline]
    pub fn bg_rgb(&mut self, color: Rgb) -> std::io::Result<()> {
        if self.inverted {
            self.raw_fg_rgb(color)
        } else {
            self.raw_bg_rgb(color)
        }
    }

    #[inline]
    pub fn fg16(&mut self, color: Color16) -> std::io::Result<()> {
        if self.inverted {
            self.raw_bg16(color)
        } else {
            self.raw_fg16(color)
        }
    }

    #[inline]
    pub fn bg16(&mut self, color: Color16) -> std::io::Result<()> {
        if self.inverted {
            self.raw_fg16(color)
        } else {
            self.raw_bg16(color)
        }
    }

    #[inline]
    pub fn fg(&mut self, color: Color) -> std::io::Result<()> {
        match color {
            Color::Default => self.fg_default(),
            Color::Rgb { r, g, b } => self.fg_rgb(Rgb { r, g, b }),
            Color::Color16(color) => self.fg16(color),
        }
    }

    #[inline]
    pub fn bg(&mut self, color: Color) -> std::io::Result<()> {
        match color {
            Color::Default => self.bg_default(),
            Color::Rgb { r, g, b } => self.bg_rgb(Rgb { r, g, b }),
            Color::Color16(color) => self.bg16(color),
        }
    }

    pub fn rich_text(&mut self, row: i32, column: i32, rich_text: &RichText) -> std::io::Result<()> {
        self.rich_text_cropped(
            row, column,
            rich_text.width().min(u32::MAX as usize) as u32,
            rich_text.height().min(u32::MAX as usize) as u32,
            rich_text,
        )
    }

    pub fn rich_text_cropped(&mut self, row: i32, column: i32, width: u32, height: u32, rich_text: &RichText) -> std::io::Result<()> {
        let min_height = rich_text.height().min(height as usize);
        let min_width = rich_text.width().min(width as usize);

        if row < 0 && -row as usize >= min_height {
            return Ok(());
        }

        if column < 0 && -column as usize >= min_width {
            return Ok(());
        }

        if row > 0 && row as u32 > self.window_size.rows {
            return Ok(());
        }

        if column > 0 && column as u32 > self.window_size.columns {
            return Ok(());
        }

        let start_line_index = if row < 0 { -row as usize } else { 0 };
        let end_line_index = (start_line_index + height.min(self.window_size.rows) as usize).min(rich_text.height());

        // {
        //     let mut f = std::fs::OpenOptions::new().append(true).create(true).open("tmp/error.log")?;
        //     writeln!(f, ">>> [x{}][{}..{}] row: {row}, height: {height}", rich_text.height(), start_line_index, end_line_index)?;
        // }

        let lines = &rich_text.lines()[start_line_index..end_line_index];

        self.clear_style()?;

        if !self.default_fg.is_default() {
            self.raw_fg_default()?;
        }

        if !self.default_bg.is_default() {
            self.raw_bg_default()?;
        }

        let start_row = if row < 0 { 0 } else { row as u32 };
        let start_column = if column < 0 { 0 } else { column as u32 };

        for (line_index, line) in lines.iter().enumerate() {
            let mut moved = false;
            let mut line_width = 0;
            let start_width = if column < 0 { -column as usize } else { 0 };
            let end_width = if column < 0 {
                (width as usize - -column as usize).min(self.window_size.columns as usize + -column as usize)
            } else {
                (width as usize).min(self.window_size.columns as usize - column as usize)
            };

            for code in line {
                match code {
                    RichTextCode::FontWeight(font_weight) => self.font_weight(*font_weight)?,
                    RichTextCode::FontStyle(font_style) => self.font_style(*font_style)?,
                    RichTextCode::TextDecoration(text_decoration) => self.text_decoration(*text_decoration)?,
                    RichTextCode::Foreground(color) => self.fg(*color)?,
                    RichTextCode::Background(color) => self.bg(*color)?,
                    RichTextCode::Text { text, width: text_width } => {
                        if line_width >= start_width && line_width + text_width <= end_width {
                            if !moved {
                                self.move_cursor(start_row + line_index as u32, start_column)?;
                                moved = true;
                            }

                            self.write_str(text)?;
                        } else if line_width + text_width >= start_width && line_width < end_width {
                            if !moved {
                                self.move_cursor(start_row + line_index as u32, start_column)?;
                                moved = true;
                            }

                            let text_start_width = if line_width >= start_width { 0 } else { start_width - line_width };
                            let text = crop(
                                text,
                                text_start_width,
                                end_width - text_start_width,
                            );
                            self.write_str(text)?;
                        }

                        line_width += text_width;
                    }
                }
            }
        }

        Ok(())
    }

    #[inline]
    pub fn clear_style(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[0m")
    }

    #[inline]
    pub fn bold(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[1m")
    }

    #[inline]
    pub fn faint(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[2m")
    }

    #[inline]
    pub fn italic(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[3m")
    }

    #[inline]
    pub fn not_italic(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[23m")
    }

    #[inline]
    pub fn underline(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[4m")
    }

    #[inline]
    pub fn doubly_underline(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[21m")
    }

    #[inline]
    pub fn normal_intensity(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[22m")
    }

    #[inline]
    pub fn not_underline(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[24m")
    }

    pub fn font_weight(&mut self, font_weight: FontWeight) -> std::io::Result<()> {
        match font_weight {
            FontWeight::Normal => self.normal_intensity(),
            FontWeight::Bold => self.bold(),
            FontWeight::Faint => self.faint(),
        }
    }

    pub fn text_decoration(&mut self, text_decoration: TextDecoration) -> std::io::Result<()> {
        match text_decoration {
            TextDecoration::None => self.not_underline(),
            TextDecoration::Underline => self.underline(),
            TextDecoration::DoublyUnderline => self.doubly_underline(),
        }
    }

    pub fn font_style(&mut self, font_style: FontStyle) -> std::io::Result<()> {
        match font_style {
            FontStyle::Normal => self.not_italic(),
            FontStyle::Italic => self.italic(),
        }
    }

    pub fn text_style(&mut self, style: &TextStyle) -> std::io::Result<()> {
        if let Some(font_weight) = style.font_weight {
            self.font_weight(font_weight)?;
        }

        if let Some(text_decoration) = style.text_decoration {
            self.text_decoration(text_decoration)?;
        }

        if let Some(font_style) = style.font_style {
            self.font_style(font_style)?;
        }

        if let Some(fg) = style.foreground {
            self.fg(fg)?;
        }

        if let Some(bg) = style.background {
            self.fg(bg)?;
        }

        Ok(())
    }

    #[inline]
    pub fn invert(&mut self) {
        self.inverted = !self.inverted
    }

    #[inline]
    pub fn set_inverted(&mut self, inverted: bool) {
        self.inverted = inverted;
    }

    #[inline]
    pub fn inverted(&self) -> bool {
        self.inverted
    }

    #[inline]
    pub fn move_cursor(&mut self, row: u32, column: u32) -> std::io::Result<()> {
        write!(self.writer, "\x1B[{row};{column}H")
    }

    #[inline]
    pub fn move_cursor_up(&mut self, amount: u32) -> std::io::Result<()> {
        write!(self.writer, "\x1B[{amount}A")
    }

    #[inline]
    pub fn move_cursor_down(&mut self, amount: u32) -> std::io::Result<()> {
        write!(self.writer, "\x1B[{amount}B")
    }

    #[inline]
    pub fn move_cursor_forward(&mut self, amount: u32) -> std::io::Result<()> {
        write!(self.writer, "\x1B[{amount}C")
    }

    #[inline]
    pub fn move_cursor_back(&mut self, amount: u32) -> std::io::Result<()> {
        write!(self.writer, "\x1B[{amount}D")
    }

    #[inline]
    pub fn clear_screen(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[2J")
    }

    #[inline]
    pub fn clear_line(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[2K")
    }

    #[inline]
    pub fn clear_line_to_end(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[0K")
    }

    #[inline]
    pub fn clear_line_to_start(&mut self) -> std::io::Result<()> {
        self.writer.write_all(b"\x1B[1K")
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

    #[inline]
    pub fn window_size(&self) -> &WindowSize {
        &self.window_size
    }

    pub fn refresh_window_size(&mut self) -> std::io::Result<&WindowSize> {
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

        self.window_size = WindowSize {
            rows: winsize.ws_row.into(),
            columns: winsize.ws_col.into(),
        };

        return Ok(&self.window_size);
    }

    pub fn enable_mouse(&mut self) -> std::io::Result<()> {
        // https://c-for-dummies.com/blog/?p=7363
        self.write(b"\x1B[?1000h\x1B[?1003h\x1B[?1006h")?;
        self.mouse_enabled = true;
        Ok(())
    }

    pub fn disable_mouse(&mut self) -> std::io::Result<()> {
        self.write(b"\x1B[?1001l")?;
        self.mouse_enabled = false;
        Ok(())
    }

    #[inline]
    pub fn is_mouse_enabled(&self) -> bool {
        self.mouse_enabled
    }

    pub fn wait(&mut self) -> std::io::Result<Option<Event>> {
        let sigwinch_nr = SIGWINCH_NR.load(Ordering::Relaxed);
        if self.sigwinch_nr != sigwinch_nr {
            self.sigwinch_nr = sigwinch_nr;

            let old_window_size = self.window_size;
            let new_window_size = self.refresh_window_size()?;
            if *new_window_size != old_window_size {
                return Ok(Some(Event::WindowSize {
                    rows: new_window_size.rows,
                    columns: new_window_size.columns,
                }));
            }
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

                            let old_window_size = self.window_size;
                            let new_window_size = self.refresh_window_size()?;
                            if *new_window_size != old_window_size {
                                return Ok(Some(Event::WindowSize {
                                    rows: new_window_size.rows,
                                    columns: new_window_size.columns,
                                }));
                            }
                        }
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }

    fn parse_utf8(&mut self, byte: u8) -> std::io::Result<Option<char>> {
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

                            return Ok(Some(unsafe { char::from_u32_unchecked(codepoint) }));
                        } else {
                            self.unget_slice(&[b2, b3]);
                            return Ok(Some(surrogate_escape(byte)));
                        }
                    } else {
                        self.unget_byte(b2);
                        return Ok(Some(surrogate_escape(byte)));
                    }
                } else {
                    return Ok(Some(surrogate_escape(byte)));
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

                        return Ok(Some(unsafe { char::from_u32_unchecked(codepoint) }));
                    } else {
                        self.unget_byte(b2);
                        return Ok(Some(surrogate_escape(byte)));
                    }
                } else {
                    return Ok(Some(surrogate_escape(byte)));
                }
            } else {
                // 2 bytes
                codepoint &= 0x1F;

                let b2 = self.peek_byte()?;

                if let Some(b2) = b2 && is_cont(b2) {
                    self.buffer_index += 1;

                    codepoint <<= 6;
                    codepoint |= b2 as u32 & 0x3F;

                    return Ok(Some(unsafe { char::from_u32_unchecked(codepoint) }));
                } else {
                    return Ok(Some(surrogate_escape(byte)));
                }
            }
        }

        return Ok(Some(byte.into()));
    }

    pub fn read(&mut self) -> std::io::Result<Option<Event>> {
        let sigwinch_nr = SIGWINCH_NR.load(Ordering::Relaxed);
        if self.sigwinch_nr != sigwinch_nr {
            self.sigwinch_nr = sigwinch_nr;

            let old_window_size = self.window_size;
            let new_window_size = self.refresh_window_size()?;
            if *new_window_size != old_window_size {
                return Ok(Some(Event::WindowSize {
                    rows: new_window_size.rows,
                    columns: new_window_size.columns,
                }));
            }
        }

        let Some(byte) = self.read_byte()? else {
            return Ok(None);
        };

        if byte == b'\r' {
            return Ok(Some(Event::KeyPress { ctrl: false, alt: false, shift: false, key: Key::Enter }));
        }

        if byte >= 1 && byte <= 26 {
            return Ok(Some(Event::KeyPress { alt: false, ctrl: true, shift: false, key: Key::Char((b'a' + byte).into()) }));
        }

        if byte != ESCAPE {
            let ch = self.parse_utf8(byte)?;
            return Ok(ch.map(Event::from_char));
        }

        // else ESC
        // https://en.wikipedia.org/wiki/ANSI_escape_code#Terminal_input_sequences

        let Some(byte) = self.peek_byte()? else {
            return Ok(Some(ESCAPE_EVENT));
        };

        if byte == b'[' {
            // CSI
            self.buffer_index += 1;

            let Some(mut byte) = self.peek_byte()? else {
                return Ok(Some(Event::alt_key(Key::Char('['))));
            };

            if byte == ESCAPE {
                // guess this is a new escape sequence and it was just Alt+[
                return Ok(Some(Event::alt_key(Key::Char('['))));
            }

            match byte {
                b'A' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Up))); }
                b'B' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Down))); }
                b'C' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Right))); }
                b'D' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Left))); }
                b'F' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::End))); }
                b'E' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Keypad5))); }
                b'G' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Keypad5))); }
                b'H' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Home))); }
                b'P' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Pause))); }
                b'[' => { // '\x1B[['
                    self.buffer_index += 1;

                    let Some(next) = self.peek_byte()? else {
                        self.unget_byte(b'[');
                        return Ok(Some(Event::alt_key(Key::Char('['))));
                    };

                    match next {
                        b'A' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Function(1)))); }
                        b'B' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Function(2)))); }
                        b'C' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Function(3)))); }
                        b'D' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Function(4)))); }
                        b'E' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Function(5)))); }
                        _ => { return Ok(Some(Event::Unsupported)); }
                    }
                }
                _ => {}
            }

            let flag = if byte == b'?' || byte == b'<' {
                let flag = byte;
                self.buffer_index += 1;

                let Some(next) = self.peek_byte()? else {
                    return Ok(Some(Event::Unsupported));
                };

                byte = next;

                flag
            } else {
                0
            };

            if byte.is_ascii_digit() {
                self.uint_buffer.clear();

                loop {
                    let mut uint: u32 = 0;
                    while byte.is_ascii_digit() {
                        // check for integer overflow?
                        uint *= 10;
                        uint += (byte - b'0') as u32;

                        self.buffer_index += 1;
                        let Some(next) = self.peek_byte()? else {
                            break;
                        };

                        byte = next;
                    }

                    self.uint_buffer.push(uint);

                    if byte != b';' {
                        break;
                    }

                    self.buffer_index += 1;
                    let Some(next) = self.peek_byte()? else {
                        break;
                    };

                    byte = next;
                }
            }

            self.buffer_index += 1;

            match (flag, byte, self.uint_buffer.len()) {
                (0, b'~', 0) => {
                    // no number defaults to 1
                    return Ok(Some(Event::key(Key::Home)));
                }
                (0, b'~', 1) => {
                    match self.uint_buffer[0] {
                        2 => { return Ok(Some(Event::key(Key::Insert))); }
                        3 => { return Ok(Some(Event::key(Key::Delete))); }
                        4 => { return Ok(Some(Event::key(Key::End))); }
                        5 => { return Ok(Some(Event::key(Key::PageUp))); }
                        6 => { return Ok(Some(Event::key(Key::PageDown))); }
                        7 => { return Ok(Some(Event::key(Key::Home))); }
                        8 => { return Ok(Some(Event::key(Key::End))); }

                        15 => { return Ok(Some(Event::key(Key::Function(5)))); }

                        17 => { return Ok(Some(Event::key(Key::Function(6)))); }
                        18 => { return Ok(Some(Event::key(Key::Function(7)))); }
                        19 => { return Ok(Some(Event::key(Key::Function(8)))); }
                        20 => { return Ok(Some(Event::key(Key::Function(9)))); }
                        21 => { return Ok(Some(Event::key(Key::Function(10)))); }

                        23 => { return Ok(Some(Event::key(Key::Function(11)))); }
                        24 => { return Ok(Some(Event::key(Key::Function(12)))); }
                        25 => { return Ok(Some(Event::key(Key::Function(13)))); }
                        26 => { return Ok(Some(Event::key(Key::Function(14)))); }

                        28 => { return Ok(Some(Event::key(Key::Function(15)))); }
                        29 => { return Ok(Some(Event::key(Key::Function(16)))); }

                        31 => { return Ok(Some(Event::key(Key::Function(17)))); }
                        32 => { return Ok(Some(Event::key(Key::Function(18)))); }
                        33 => { return Ok(Some(Event::key(Key::Function(19)))); }
                        34 => { return Ok(Some(Event::key(Key::Function(20)))); }

                        _ => {
                            return Ok(Some(Event::Unsupported));
                        }
                    }
                }
                (b'<', b'M', 3) | (b'<', b'm', 3) => {
                    // mouse
                    let _release = byte == b'm';
                    let _buttons = self.uint_buffer[0];
                    let _column = self.uint_buffer[1];
                    let _row = self.uint_buffer[2];

                    // TODO: mouse support
                    return Ok(Some(Event::Unsupported));
                }
                (0, b'R', 2) | (b'?', b'R', 2) => {
                    return Ok(Some(Event::CursorPosition { row: self.uint_buffer[0], column: self.uint_buffer[1] }));
                }
                _ => {
                    return Ok(Some(Event::Unsupported));
                }
            }
        } else if byte == b'O' {
            // SS3
            self.buffer_index += 1;

            let Some(next) = self.read_byte()? else {
                return Ok(Some(Event::KeyPress { alt: true, ctrl: false, shift: true, key: Key::Char('O') }));
            };

            match next {
                b'P' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Function(1)))); }
                b'Q' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Function(2)))); }
                b'R' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Function(3)))); }
                b'S' => { self.buffer_index += 1; return Ok(Some(Event::key(Key::Function(4)))); }
                b'M' => { self.buffer_index += 1; return Ok(Some(Event::KeyPress { ctrl: false, alt: false, shift: true, key: Key::Enter })); }
                _ => {}
            }
        }

        self.buffer_index += 1;

        if byte == ESCAPE && let Some(next) = self.peek_byte()? && next == b'O' {
            self.buffer_index += 1;

            if let Some(next) = self.peek_byte()? && next == b'M' {
                self.buffer_index += 1;
                return Ok(Some(Event::KeyPress { ctrl: false, alt: true, shift: true, key: Key::Enter }));
            } else {
                return Ok(Some(Event::Unsupported));
            }
        }

        let ch = self.parse_utf8(byte)?;
        return Ok(ch.map(Event::from_char_alt));
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

        if self.mouse_enabled {
            let _ = self.disable_mouse();
        }

        // CSI 0 m        Reset or normal, all attributes become turned off
        // CSI ?   25 h   Show cursor (DECTCEM), VT220
        // CSI ?    7 h   Auto-Wrap Mode (DECAWM), VT100
        // CSI ? 1049 l   Disable alternative screen buffer
        let _ = self.write(b"\x1B[0m\x1B[?25h\x1B[?7h\x1B[?1049l");
        let _ = self.flush();

        if self.wfd != libc::STDOUT_FILENO && self.wfd != libc::STDERR_FILENO {
            let _ = unsafe { libc::close(self.wfd) };
        }

        if self.rfd != libc::STDIN_FILENO && self.wfd != self.rfd {
            let _ = unsafe { libc::close(self.rfd) };
        }
    }
}
