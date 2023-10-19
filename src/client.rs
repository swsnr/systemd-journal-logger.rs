// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A journald client.

use std::fs::File;
use std::io::prelude::*;
use std::os::fd::AsFd;
use std::os::unix::net::UnixDatagram;

use rustix::fs::fcntl_add_seals;
use rustix::fs::memfd_create;
use rustix::fs::MemfdFlags;
use rustix::fs::SealFlags;
use rustix::io::Errno;
use rustix::net::sendmsg_unix;
use rustix::net::SendAncillaryBuffer;
use rustix::net::SendFlags;
use rustix::net::SocketAddrUnix;

const JOURNALD_PATH: &str = "/run/systemd/journal/socket";

pub struct JournalClient {
    socket: UnixDatagram,
}

impl JournalClient {
    pub fn new() -> std::io::Result<Self> {
        let client = Self {
            socket: UnixDatagram::unbound()?,
        };
        // Check that we can talk to journald, by sending empty payload which journald discards.
        // However if the socket didn't exist or if none listened we'd get an error here.
        client.send_payload(&[])?;
        Ok(client)
    }

    /// Send `payload` to journald.
    ///
    /// Directly send it as datagram, and fall back to [`Self::send_large_payload`]
    /// if that fails with `EMSGSIZE`.
    pub fn send_payload(&self, payload: &[u8]) -> std::io::Result<usize> {
        self.socket
            .send_to(payload, JOURNALD_PATH)
            .or_else(|error| {
                if Some(Errno::MSGSIZE) == Errno::from_io_error(&error) {
                    self.send_large_payload(payload)
                } else {
                    Err(error)
                }
            })
    }

    /// Send a large payload to journald.
    ///
    /// Write payload to a memfd, seal it, and then send the FD to the socket in
    /// an ancilliary message.
    ///
    /// See <https://systemd.io/JOURNAL_NATIVE_PROTOCOL/>.
    fn send_large_payload(&self, payload: &[u8]) -> std::io::Result<usize> {
        let mut mem: File = memfd_create(
            "systemd-journal-logger",
            MemfdFlags::ALLOW_SEALING | MemfdFlags::CLOEXEC,
        )?
        .into();
        mem.write_all(payload)?;
        // Fully seal the memfd to signal journald that it is safe to mmap now.
        fcntl_add_seals(
            &mem,
            SealFlags::SEAL | SealFlags::SHRINK | SealFlags::WRITE | SealFlags::GROW,
        )?;
        let fds = &[mem.as_fd()];
        let scm_rights = rustix::net::SendAncillaryMessage::ScmRights(fds);
        // We use a static buffer size here, because we don't need to account
        // for arbitrary messages; we just need enough space for a single FD.
        // With a static buffer we get away without any additional heap
        // allocations here.
        let mut buffer = [0; 64];
        // Just a sanity check should we ever get the static buffer size wrong.
        assert!(
            scm_rights.size() <= buffer.len(),
            "static buffer size not sufficient for ScmRights message of size {}",
            scm_rights.size()
        );
        let mut buffer = SendAncillaryBuffer::new(&mut buffer);
        // push returns false if the buffer is too small to add the new message;
        // let's guard against this.
        assert!(buffer.push(scm_rights), "Failed to push ScmRights message");
        let size = sendmsg_unix(
            &self.socket,
            &SocketAddrUnix::new(JOURNALD_PATH)?,
            &[],
            &mut buffer,
            SendFlags::NOSIGNAL,
        )?;
        Ok(size)
    }
}
