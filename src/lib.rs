#[cfg_attr(unix, path = "sys/unix.rs")]
#[cfg_attr(windows, path = "sys/windows.rs")]
mod sys;

use std::fs::File;
use std::io::Result;

use crate::sys::{borrow_fd, override_stdio, AsFd, BorrowedFd, OwnedFd, DEV_NULL};

#[derive(Clone, Copy)]
enum Stdio {
    Stdin,
    Stdout,
    Stderr,
}

pub trait StdioOverride: AsFd {
    /// Replace the process standard output with a duplicate of the file descriptor.
    ///
    /// ```rust
    /// # use stdio_utils::StdioOverride as _;
    /// # use std::fs::{File, read_to_string};
    /// # use std::io::stdout;
    /// # let mut lock = stdout().lock();
    /// let _guard = File::create("./output.txt")?.override_stdout()?;
    /// println!("hello world!");
    /// let output = read_to_string("./output.txt")?;
    /// assert_eq!(output, "hello world!\n");
    /// # std::io::Result::Ok(())
    /// ```
    fn override_stdout(&self) -> Result<Guard> {
        let stdio = Stdio::Stdout;
        let backup = unsafe { override_stdio(self, stdio) }?;
        let backup = Some(backup);
        Ok(Guard { backup, stdio })
    }

    /// Replace the process standard error with a duplicate of the file descriptor.
    ///
    /// See [duplicate_to_stdout](StdioOverride::override_stdout).
    fn override_stderr(&self) -> Result<Guard> {
        let stdio = Stdio::Stderr;
        let backup = unsafe { override_stdio(self, stdio) }?;
        let backup = Some(backup);
        Ok(Guard { backup, stdio })
    }

    /// Replace the process standard input with a duplicate of the file descriptor.
    ///
    /// ```rust
    /// # use stdio_utils::StdioOverride as _;
    /// # use std::fs::{File, write};
    /// # use std::io::{stdin, stdout, read_to_string};
    /// # let mut lock = stdout().lock();
    /// write("./input.txt", "hello world!")?;
    /// let _guard = File::open("./input.txt")?.override_stdin()?;
    /// let input = read_to_string(stdin())?;
    /// assert_eq!(input, "hello world!");
    /// # std::io::Result::Ok(())
    /// ```
    fn override_stdin(&self) -> Result<Guard> {
        let stdio = Stdio::Stdin;
        let backup = unsafe { override_stdio(self, stdio) }?;
        let backup = Some(backup);
        Ok(Guard { backup, stdio })
    }
}

impl<T: AsFd> StdioOverride for T {}

#[must_use]
/// A type that restores a replaced file descriptor when it's dropped
pub struct Guard {
    stdio: Stdio,
    backup: Option<OwnedFd>,
}

impl Guard {
    /// Consume the guard without restoring the file descriptor.
    pub fn forget(self) {
        self.into_inner();
    }

    /// Consume the guard returning an OwnedFd with the original file descriptor
    pub fn into_inner(mut self) -> OwnedFd {
        self.backup.take().unwrap()
    }

    /// Obtain a BorrowFd to the original file descriptor
    pub fn borrow_inner(&self) -> BorrowedFd<'_> {
        borrow_fd(self.backup.as_ref().unwrap())
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if let Some(backup) = self.backup.take() {
            let _ = unsafe { override_stdio(backup, self.stdio) };
        }
    }
}

/// Returns a file that can be written to.
/// The file discards all bytes written to it.
///
/// ```rust
/// # use stdio_utils::null;
/// # use std::io::Write;
/// let mut f = null()?;
/// f.write_all(b"hello world")?;
/// # std::io::Result::Ok(())
/// ```
///
/// It can be used to dicard all the data written
/// to stdout
///
/// ```rust
/// # use stdio_utils::{null, StdioOverride as _};
/// # use std::io::Write;
/// # use std::io::stdout;
/// # let mut lock = stdout().lock();
/// let _guard = null()?.override_stdout();
/// println!("hello world!"); // this will never print
/// # std::io::Result::Ok(())
/// ```
pub fn null() -> Result<File> {
    File::options()
        .create(false)
        .read(false)
        .append(true)
        .open(DEV_NULL)
}
