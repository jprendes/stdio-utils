#[cfg_attr(unix, path = "sys/unix.rs")]
#[cfg_attr(windows, path = "sys/windows.rs")]
mod sys;

use std::fs::File;
use std::io::{stderr, stdin, stdout, Result};

use sys::DEV_NULL;

use crate::sys::{Owned, RawFd, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};

mod private {
    pub trait Sealed {}
}

pub trait Duplicate: private::Sealed {
    #[doc(hidden)]
    fn duplicate(&self) -> Result<Owned>;

    #[doc(hidden)]
    unsafe fn duplicate_to_fd(&self, dst: RawFd) -> Result<()>;

    /// Replace the process standard output with a duplicate of the file descriptor.
    ///
    /// ```rust
    /// # use stdio_utils::Duplicate as _;
    /// # use std::fs::{File, read_to_string};
    /// # use std::io::stdout;
    /// # let mut lock = stdout().lock();
    /// File::create("./output.txt")?.duplicate_to_stdout()?;
    /// println!("hello world!");
    /// let output = read_to_string("./output.txt")?;
    /// assert_eq!(output, "hello world!\n");
    /// # std::io::Result::Ok(())
    /// ```
    fn duplicate_to_stdout(&self) -> Result<()>;

    /// Replace the process standard error with a duplicate of the file descriptor.
    ///
    /// See [duplicate_to_stdout](Duplicate::duplicate_to_stdout).
    fn duplicate_to_stderr(&self) -> Result<()>;

    /// Replace the process standard input with a duplicate of the file descriptor.
    ///
    /// ```rust
    /// # use stdio_utils::Duplicate as _;
    /// # use std::fs::{File, write};
    /// # use std::io::{stdin, stdout, read_to_string};
    /// # let mut lock = stdout().lock();
    /// write("./input.txt", "hello world!")?;
    /// File::open("./input.txt")?.duplicate_to_stdin()?;
    /// let input = read_to_string(stdin())?;
    /// assert_eq!(input, "hello world!");
    /// # std::io::Result::Ok(())
    /// ```
    fn duplicate_to_stdin(&self) -> Result<()>;
}

/// A type that restores a replaced file descriptor when it's dropped
pub struct Guard {
    fd: RawFd,
    backup: Owned,
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = unsafe { self.backup.duplicate_to_fd(self.fd) };
    }
}

impl Guard {
    fn new(owned: impl Duplicate, fd: RawFd) -> Result<Self> {
        let backup = owned.duplicate()?;
        Ok(Self { backup, fd })
    }

    /// Restore stdin to it's original file when the `Guard` is dropped.
    ///
    /// ```rust
    /// # use stdio_utils::{Guard, null, Duplicate};
    /// # use std::fs::{File, write};
    /// # use std::io::{stdin, stdout, read_to_string};
    /// # let mut lock = stdout().lock();
    /// write("./input.txt", "hello world!")?;
    /// File::open("./input.txt")?.duplicate_to_stdin()?;
    ///
    /// let guard = Guard::stdin();
    /// null()?.duplicate_to_stdin()?;
    ///
    /// // reading from null errors on windows
    /// let input = read_to_string(stdin()).unwrap_or_default();
    /// assert_eq!(input, "");
    ///
    /// drop(guard);
    ///
    /// let input = read_to_string(stdin())?;
    /// assert_eq!(input, "hello world!");
    /// # std::io::Result::Ok(())
    /// ```
    pub fn stdin() -> Result<Self> {
        Self::new(stdin(), STDIN_FILENO)
    }

    /// Restore stdout to it's original file when the `Guard` is dropped.
    ///
    /// ```rust
    /// # use stdio_utils::{Guard, null, Duplicate};
    /// # use std::io::stdout;
    /// # let mut lock = stdout().lock();
    /// let guard = Guard::stdout()?;
    /// null()?.duplicate_to_stdout()?;
    /// println!("hello hidden world!"); // This won't be printed
    /// drop(guard);
    /// println!("hello visible world!"); // This will be printed
    /// # std::io::Result::Ok(())
    /// ```
    pub fn stdout() -> Result<Self> {
        Self::new(stdout(), STDOUT_FILENO)
    }

    /// Restore stderr to it's original file when the `Guard` is dropped.
    ///
    /// See [stdout](Guard::stdout).
    pub fn stderr() -> Result<Self> {
        Self::new(stderr(), STDERR_FILENO)
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
/// # use stdio_utils::{null, Duplicate};
/// # use std::io::Write;
/// # use std::io::stdout;
/// # let mut lock = stdout().lock();
/// null()?.duplicate_to_stdout();
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
