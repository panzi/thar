use std::{os::fd::RawFd, time::Duration};

use bitflags::bitflags;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Event(libc::epoll_event);

impl Event {
    #[inline]
    pub fn new(events: Events, u64: u64) -> Self {
        Self(libc::epoll_event { events: events.bits(), u64 })
    }

    #[inline]
    pub fn events(&self) -> Events {
        Events(self.0.events.into())
    }

    #[inline]
    pub fn u64(&self) -> u64 {
        self.0.u64
    }
}

impl Default for Event {
    #[inline]
    fn default() -> Self {
        Self(libc::epoll_event { events: 0, u64: 0 })
    }
}

#[derive(Debug)]
pub struct EPoll {
    fd: RawFd,
}

impl EPoll {
    #[inline]
    pub fn new() -> std::io::Result<Self> {
        EPoll::with_flags(CreateFlags::CloseOnExec)
    }

    pub fn with_flags(flags: CreateFlags) -> std::io::Result<Self> {
        let fd = unsafe { libc::epoll_create1(flags.bits()) };

        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(Self { fd })
    }

    #[inline]
    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn add(&mut self, fd: RawFd, events: Events, u64: u64) -> std::io::Result<()> {
        let mut event = libc::epoll_event {
            events: events.bits(),
            u64
        };

        let res = unsafe { libc::epoll_ctl(self.fd, libc::EPOLL_CTL_ADD, fd, &mut event) };

        if res < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn modify(&mut self, fd: RawFd, events: Events, u64: u64) -> std::io::Result<()> {
        let mut event = libc::epoll_event {
            events: events.bits(),
            u64
        };

        let res = unsafe { libc::epoll_ctl(self.fd, libc::EPOLL_CTL_MOD, fd, &mut event) };

        if res < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn delete(&mut self, fd: RawFd) -> std::io::Result<()> {
        let res = unsafe { libc::epoll_ctl(self.fd, libc::EPOLL_CTL_DEL, fd, std::ptr::null_mut()) };

        if res < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn wait(&mut self, events: &mut [Event], timeout: Option<Duration>) -> std::io::Result<usize> {
        let maxevents = events.len();
        if maxevents > i32::MAX as usize {
            return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
        }

        let timeout = if let Some(timeout) = timeout {
            if timeout.as_secs() > i64::MAX as u64 {
                return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
            }

            &libc::timespec {
                tv_sec:  timeout.as_secs() as i64,
                tv_nsec: timeout.subsec_nanos() as i64
            }
        } else {
            std::ptr::null()
        };

        let res = unsafe {
            libc::epoll_pwait2(
                self.fd,
                events.as_mut_ptr() as *mut libc::epoll_event,
                maxevents as i32,
                timeout,
                std::ptr::null()
            )
        };

        if res < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(res as usize)
    }
}

impl Drop for EPoll {
    #[inline]
    fn drop(&mut self) {
        unsafe { libc::close(self.fd); }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CreateFlags: libc::c_int {
        const CloseOnExec = libc::EPOLL_CLOEXEC;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Events: u32 {
        // once it is in stable use #[cfg(accessible(libc::EPOLLRDHUP))]
        // though that feature is in limbo since many years now, so it probably won't ever happen

        const In            = libc::EPOLLIN        as u32;
        const Out           = libc::EPOLLOUT       as u32;
        const ReadHangup    = libc::EPOLLRDHUP     as u32;
        const Priority      = libc::EPOLLPRI       as u32;
        const Error         = libc::EPOLLERR       as u32;
        const Hangup        = libc::EPOLLHUP       as u32;
        const EdgeTriggered = libc::EPOLLET        as u32;
        const OneShot       = libc::EPOLLONESHOT   as u32;
        const Wakeup        = libc::EPOLLWAKEUP    as u32;
        const Exclusive     = libc::EPOLLEXCLUSIVE as u32;
    }
}
