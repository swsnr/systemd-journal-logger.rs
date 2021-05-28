// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::process::Command;

use log::{warn, LevelFilter};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct JournalEntry {
    priority: String,
    message: String,
    target: String,
    code_file: String,
    code_line: String,
    rust_module_path: String,
    syslog_identifier: String,
    syslog_pid: String,
    #[serde(rename = "_PID")]
    pid: String,
}

#[test]
fn log_to_journal() {
    systemd_journal_logger::init().unwrap();

    log::set_max_level(LevelFilter::Info);

    warn!(target: "systemd_journal_logger/log_to_journal", "systemd_journal_logger test: {}", 42);

    let stdout = String::from_utf8(
        Command::new("journalctl")
            .args(&["--user", "--output=json"])
            // Filter by the PID of the current test process and the module path
            .arg(format!("_PID={}", std::process::id()))
            .arg(format!("RUST_MODULE_PATH={}", module_path!()))
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let messages: Vec<&str> = stdout.lines().collect();
    assert_eq!(messages.len(), 1);

    let entry: JournalEntry = serde_json::from_str(messages[0]).unwrap();

    assert_eq!(
        entry.priority.parse::<u8>().unwrap(),
        u8::from(libsystemd::logging::Priority::Warning)
    );
    assert_eq!(entry.message, "systemd_journal_logger test: 42");
    assert_eq!(entry.code_file, file!());
    assert_eq!(entry.code_line, "35");
    assert_eq!(entry.rust_module_path, module_path!());
    assert_eq!(entry.target, "systemd_journal_logger/log_to_journal");

    assert!(entry.syslog_identifier.contains("log_to_journal"));
    assert_eq!(
        entry.syslog_identifier,
        std::env::current_exe()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    );

    assert_eq!(entry.syslog_pid, std::process::id().to_string());
    // The PID we logged is equal to the PID systemd determined as source for our process
    assert_eq!(entry.syslog_pid, entry.pid);
}
