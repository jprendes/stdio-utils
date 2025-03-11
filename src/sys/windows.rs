use std::io::{Error, Result};
pub(crate) use std::os::windows::io::{
    AsHandle as AsFd, BorrowedHandle as BorrowedFd, OwnedHandle as OwnedFd,
};
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle};

use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::System::Console::*;

use crate::{AsFdExt as _, Stdio};

pub(crate) const DEV_NULL: &str = "nul";

impl Stdio {
    fn id(&self) -> STD_HANDLE {
        match self {
            Stdio::Stdin => STD_INPUT_HANDLE,
            Stdio::Stdout => STD_OUTPUT_HANDLE,
            Stdio::Stderr => STD_ERROR_HANDLE,
        }
    }

    fn as_raw_handle(&self) -> Result<RawHandle> {
        let handle = unsafe { GetStdHandle(self.id()) };
        if handle == INVALID_HANDLE_VALUE {
            return Err(Error::last_os_error());
        }
        Ok(handle)
    }

    unsafe fn set_raw_handle(&self, handle: RawHandle) -> Result<()> {
        if SetStdHandle(self.id(), handle) == 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
}

pub(crate) fn borrow_fd(file: &(impl AsFd + ?Sized)) -> BorrowedFd<'_> {
    file.as_handle()
}

pub(crate) unsafe fn override_stdio(file: impl AsFd, stdio: Stdio) -> Result<OwnedFd> {
    let original = stdio.as_raw_handle()?;
    let file = file.duplicate_file()?;
    stdio.set_raw_handle(file.as_handle().as_raw_handle())?;
    let _ = file.into_raw_handle(); // drop ownership of the handle, it's managed the stdio now
    Ok(OwnedFd::from_raw_handle(original))
}
