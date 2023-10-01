// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A journald client.

use std::io::prelude::*;
use std::os::fd::AsFd;
use std::os::unix::net::UnixDatagram;

use crate::{memfd, socket};

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
                if Some(libc::EMSGSIZE) == error.raw_os_error() {
                    self.send_large_payload(payload)
                } else {
                    Err(error)
                }
            })
    }

    pub fn send_large_payload(&self, payload: &[u8]) -> std::io::Result<usize> {
        // If the payload's too large for a single datagram, send it through a memfd, see
        // https://systemd.io/JOURNAL_NATIVE_PROTOCOL/
        // Write the whole payload to a memfd
        let mut mem = memfd::create_sealable()?;
        mem.write_all(payload)?;
        // Fully seal the memfd to signal journald that its backing data won't resize anymore
        // and so is safe to mmap.
        memfd::seal_fully(mem.as_fd())?;
        socket::send_one_fd_to(&self.socket, mem.as_fd(), JOURNALD_PATH)
    }
}
