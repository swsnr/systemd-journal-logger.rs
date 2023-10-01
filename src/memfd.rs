// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Work with memfds.

// Taken from <https://github.com/tokio-rs/tracing/blob/63d41707efa98d3ce64be7fff02ee348157a6985/tracing-journald/src/memfd.rs>
// which I wrote, ie own copyright, and thus can freely relicense it.

use std::fs::File;
use std::io::{Error, Result};
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd};

use libc::{
    c_char, c_uint, fcntl, memfd_create, F_ADD_SEALS, F_SEAL_GROW, F_SEAL_SEAL, F_SEAL_SHRINK,
    F_SEAL_WRITE, MFD_ALLOW_SEALING, MFD_CLOEXEC,
};

fn create(flags: c_uint) -> Result<OwnedFd> {
    // SAFETY: memfd_create allocates a new file descriptor.  The name is a static string, so we can safely convert to a pointer.
    let fd = unsafe { memfd_create("tracing-journald\0".as_ptr() as *const c_char, flags) };
    if fd < 0 {
        Err(Error::last_os_error())
    } else {
        // SAFETY: We created fd above, so it belongs to us now.  We also checked that it's a valid fd.
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }
}

/// Create a sealable memfd.
pub fn create_sealable() -> Result<File> {
    create(MFD_ALLOW_SEALING | MFD_CLOEXEC).map(File::from)
}

pub fn seal_fully(fd: BorrowedFd) -> Result<()> {
    let all_seals = F_SEAL_SHRINK | F_SEAL_GROW | F_SEAL_WRITE | F_SEAL_SEAL;
    // SAFETY: fnctl does not take ownership of the file descriptor, so we can convert to a raw file descriptor.
    // SAFETY: We don't pass any pointers here, so there's no potential for memory issues.
    let result = unsafe { fcntl(fd.as_raw_fd(), F_ADD_SEALS, all_seals) };
    if result < 0 {
        Err(Error::last_os_error())
    } else {
        Ok(())
    }
}
