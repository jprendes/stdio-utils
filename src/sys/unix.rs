use std::io::{Error, Result};
pub(crate) use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::os::fd::{AsRawFd, RawFd};

use crate::{AsFdExt as _, Stdio};

pub(crate) const DEV_NULL: &str = "/dev/null";

impl Stdio {
    fn as_raw_fd(&self) -> RawFd {
        match self {
            Stdio::Stdin => libc::STDIN_FILENO,
            Stdio::Stdout => libc::STDOUT_FILENO,
            Stdio::Stderr => libc::STDERR_FILENO,
        }
    }

    unsafe fn set_raw_fd(&self, fd: RawFd) -> Result<()> {
        if libc::dup2(fd, self.as_raw_fd()) < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
}

impl AsFd for Stdio {
    fn as_fd(&self) -> BorrowedFd<'static> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

pub(crate) fn borrow_fd(file: &(impl AsFd + ?Sized)) -> BorrowedFd<'_> {
    file.as_fd()
}

pub(crate) unsafe fn override_stdio(file: impl AsFd, stdio: Stdio) -> Result<OwnedFd> {
    let backup = stdio.duplicate_file()?;
    stdio.set_raw_fd(file.as_fd().as_raw_fd())?;
    Ok(backup)
}
