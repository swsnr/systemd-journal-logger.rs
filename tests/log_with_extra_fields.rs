// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod journal;

use pretty_assertions::assert_eq;

#[derive(Debug)]
struct SomeDummy {
    #[allow(dead_code)]
    foo: usize,
}

#[test]
fn log_with_extra_fields() {
    systemd_journal_logger::init().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let target = journal::random_target("init");
    let error = std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "not found: the important file",
    );
    let dummy = SomeDummy { foo: 42 };

    log::error!(target: &target, dummy=log::as_debug!(dummy), spam="no eggs", error=log::as_error!(error); "Hello World");

    let entries = journal::read_current_process(module_path!(), &target);
    assert_eq!(entries.len(), 1);

    assert_eq!(entries[0]["TARGET"], target);
    assert_eq!(entries[0]["MESSAGE"], "Hello World");
    assert_eq!(entries[0]["SPAM"], "no eggs");
    assert_eq!(entries[0]["ERROR"], "not found: the important file");
    assert_eq!(entries[0]["DUMMY"], format!("{:?}", dummy));
}
