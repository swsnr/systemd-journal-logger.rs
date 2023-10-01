// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utiltities for working with sockets.

// Taken from https://github.com/tokio-rs/tracing/blob/2e84c0b2a39d7c5276ab14e757a016e948099144/tracing-journald/src/socket.rs
// which I wrote, ie own copyright, and thus can freely relicense it.

use std::io::{Error, Result};
use std::mem::{size_of, zeroed};
use std::os::fd::BorrowedFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixDatagram;
use std::os::unix::prelude::{AsRawFd, RawFd};
use std::path::Path;
use std::ptr;

use libc::*;

// We use a static buffer size, since we don't send arbitrary control messages
// here; we just need enough space to send a single file descriptor.  This is a
// lot simpler than the `CMSG_SPACE` dance.
//
// We check the size to avoid memory issues, in a test, as well as immediately
// at the beginning of `send_one_fd`.
const CMSG_BUFSIZE: usize = 64;

#[repr(C)]
union AlignedBuffer<T: Copy + Clone> {
    buffer: T,
    align: cmsghdr,
}

fn assert_cmsg_bufsize() {
    let space_one_fd = unsafe { CMSG_SPACE(size_of::<RawFd>() as u32) };
    assert!(
        space_one_fd <= CMSG_BUFSIZE as u32,
        "cmsghdr buffer too small (< {}) to hold a single fd",
        space_one_fd
    );
}

#[cfg(test)]
#[test]
fn cmsg_buffer_size_for_one_fd() {
    assert_cmsg_bufsize()
}

/// Send one file descriptor over `socket` to `path`.
///
/// Note that the FD is copied, i.e. the calling process retains ownership of
/// `fd`, and needs to take care to close it.
pub fn send_one_fd_to<P: AsRef<Path>>(
    socket: &UnixDatagram,
    fd: BorrowedFd,
    path: P,
) -> Result<usize> {
    assert_cmsg_bufsize();

    let mut addr: sockaddr_un = unsafe { zeroed() };
    let path_bytes = path.as_ref().as_os_str().as_bytes();
    // path_bytes may have at most sun_path + 1 bytes, to account for the trailing NUL byte.
    if addr.sun_path.len() <= path_bytes.len() {
        return Err(Error::from_raw_os_error(ENAMETOOLONG));
    }

    addr.sun_family = AF_UNIX as _;
    unsafe {
        std::ptr::copy_nonoverlapping(
            path_bytes.as_ptr(),
            addr.sun_path.as_mut_ptr() as *mut u8,
            path_bytes.len(),
        )
    };

    let mut msg: msghdr = unsafe { zeroed() };
    // Set the target address.
    msg.msg_name = &mut addr as *mut _ as *mut c_void;
    msg.msg_namelen = size_of::<sockaddr_un>() as socklen_t;

    // We send no data body with this message.
    msg.msg_iov = ptr::null_mut();
    msg.msg_iovlen = 0;

    // Create and fill the control message buffer with our file descriptor
    let mut cmsg_buffer = AlignedBuffer {
        buffer: ([0u8; CMSG_BUFSIZE]),
    };
    msg.msg_control = unsafe { cmsg_buffer.buffer.as_mut_ptr() as _ };
    msg.msg_controllen = unsafe { CMSG_SPACE(size_of::<RawFd>() as _) as _ };

    let cmsg: &mut cmsghdr =
        unsafe { CMSG_FIRSTHDR(&msg).as_mut() }.expect("Control message buffer exhausted");

    cmsg.cmsg_level = SOL_SOCKET;
    cmsg.cmsg_type = SCM_RIGHTS;
    cmsg.cmsg_len = unsafe { CMSG_LEN(size_of::<RawFd>() as _) as _ };

    unsafe { ptr::write(CMSG_DATA(cmsg) as *mut RawFd, fd.as_raw_fd()) };

    let result = unsafe { sendmsg(socket.as_raw_fd(), &msg, libc::MSG_NOSIGNAL) };

    if result < 0 {
        Err(Error::last_os_error())
    } else {
        // sendmsg returns the number of bytes written
        Ok(result as usize)
    }
}
