// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::process::Command;

use log::{warn, LevelFilter};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::collections::HashMap;

fn random_target(prefix: &str) -> String {
    format!(
        "{}-{}",
        prefix,
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect::<String>()
    )
}

fn read_from_journal(target: &str) -> Vec<HashMap<String, String>> {
    let stdout = String::from_utf8(
        Command::new("journalctl")
            .args(&["--user", "--output=json"])
            // Filter by the PID of the current test process and the module path
            .arg(format!("_PID={}", std::process::id()))
            .arg(format!("MODULE_PATH={}", module_path!()))
            .arg(format!("TARGET={}", target))
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn simple_log_entry() {
    systemd_journal_logger::init().ok();
    log::set_max_level(LevelFilter::Info);

    let target = random_target("systemd_journal_logger/simple_log_entry");

    warn!(target: &target, "systemd_journal_logger test: {}", 42);

    let entries = read_from_journal(&target);
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];

    assert_eq!(
        entry["PRIORITY"],
        u8::from(libsystemd::logging::Priority::Warning).to_string()
    );
    assert_eq!(entry["MESSAGE"], "systemd_journal_logger test: 42");
    assert_eq!(entry["CODE_FILE"], file!());
    assert_eq!(entry["CODE_LINE"], "55");
    assert_eq!(entry["MODULE_PATH"], module_path!());
    assert_eq!(entry["TARGET"], target);

    assert!(entry["SYSLOG_IDENTIFIER"].contains("log_to_journal"));
    assert_eq!(
        entry["SYSLOG_IDENTIFIER"],
        std::env::current_exe()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    );

    assert_eq!(entry["SYSLOG_PID"], std::process::id().to_string());
    // // The PID we logged is equal to the PID systemd determined as source for our process
    assert_eq!(entry["SYSLOG_PID"], entry["_PID"]);
}

#[test]
fn multiline_message() {
    systemd_journal_logger::init().ok();
    log::set_max_level(LevelFilter::Info);

    let target = random_target("systemd_journal_logger/multiline_message");

    warn!(
        target: &target,
        "systemd_journal_logger test\nwith\nline {}", "breaks"
    );

    let entries = read_from_journal(&target);
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];

    assert_eq!(
        entry["PRIORITY"],
        u8::from(libsystemd::logging::Priority::Warning).to_string()
    );
    assert_eq!(
        entry["MESSAGE"],
        "systemd_journal_logger test\nwith\nline breaks"
    );
    assert_eq!(entry["CODE_FILE"], file!());
    assert_eq!(entry["CODE_LINE"], "94");
    assert_eq!(entry["TARGET"], target);
}
