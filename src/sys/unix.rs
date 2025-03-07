use std::io::{Error, Result};
use std::os::fd::{AsFd, AsRawFd};
pub use std::os::fd::{OwnedFd as Owned, RawFd};

pub(crate) use libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};

pub(crate) const DEV_NULL: &str = "/dev/null";

impl<T: AsFd> crate::private::Sealed for T {}

impl<T: AsFd> crate::Duplicate for T {
    fn duplicate(&self) -> Result<Owned> {
        self.as_fd().try_clone_to_owned()
    }

    unsafe fn duplicate_to_fd(&self, dst: RawFd) -> Result<()> {
        let new = unsafe { libc::dup2(self.as_fd().as_raw_fd(), dst) };
        if new < 0 {
            return Err(Error::other("Failed to clone file descriptor"));
        }
        Ok(())
    }

    fn duplicate_to_stdout(&self) -> Result<()> {
        unsafe { self.duplicate_to_fd(STDOUT_FILENO) }
    }

    fn duplicate_to_stderr(&self) -> Result<()> {
        unsafe { self.duplicate_to_fd(STDERR_FILENO) }
    }

    fn duplicate_to_stdin(&self) -> Result<()> {
        unsafe { self.duplicate_to_fd(STDIN_FILENO) }
    }
}
