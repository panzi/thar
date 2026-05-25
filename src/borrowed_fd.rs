use std::os::fd::RawFd;


/**
 * Write to file descriptor, but don't close in drop.
 */
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorrowedFd(RawFd);

impl BorrowedFd {
    #[inline]
    pub fn new(fd: RawFd) -> Self {
        Self(fd)
    }

    #[inline]
    pub fn inner(&self) -> RawFd {
        self.0
    }

    #[inline]
    pub fn into_inner(self) -> RawFd {
        self.0
    }
}

impl From<BorrowedFd> for RawFd {
    #[inline]
    fn from(value: BorrowedFd) -> Self {
        value.0
    }
}

impl From<RawFd> for BorrowedFd {
    #[inline]
    fn from(value: RawFd) -> Self {
        Self(value)
    }
}

impl std::io::Write for BorrowedFd {
    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let count = unsafe { libc::write(self.0, buf.as_ptr() as *const libc::c_void, buf.len()) };

        if count < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(count as usize)
    }

//    #[inline]
//    fn is_write_vectored(&self) -> bool {
//        true
//    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        if bufs.len() > libc::c_int::MAX as usize {
            return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
        }

        let count = unsafe { libc::writev(self.0, bufs.as_ptr() as *const libc::iovec, bufs.len() as libc::c_int) };

        if count < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(count as usize)
    }
}
