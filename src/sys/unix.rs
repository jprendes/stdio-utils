use std::io::{Error, Result};
pub(crate) use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::os::fd::{AsRawFd, RawFd};

use crate::Stdio;

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

impl AsRawFd for Stdio {
    fn as_raw_fd(&self) -> RawFd {
        match self {
            Stdio::Stdin => libc::STDIN_FILENO,
            Stdio::Stdout => libc::STDOUT_FILENO,
            Stdio::Stderr => libc::STDERR_FILENO,
        }
    }
}

pub(crate) fn borrow_fd(file: &impl AsFd) -> BorrowedFd<'_> {
    file.as_fd()
}

pub(crate) unsafe fn override_stdio(file: impl AsFd, stdio: Stdio) -> Result<OwnedFd> {
    let fd = stdio.as_raw_fd();
    let backup = BorrowedFd::borrow_raw(fd).try_clone_to_owned()?;
    stdio.set_raw_fd(file.as_fd().as_raw_fd())?;
    Ok(backup)
}
