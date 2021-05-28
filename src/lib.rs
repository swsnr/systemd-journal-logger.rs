// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(warnings, missing_docs, clippy::all)]

//! A [log] logger for the [systemd journal][1].
//!
//! [log]: https://docs.rs/log
//! [1]: https://www.freedesktop.org/software/systemd/man/journalctl.html
//!
//! # Usage
//!
//! Use [`init`] at the beginning of your `main` function to setup journal
//! logging, configure the logging level and then use the standard macros
//! from the [`log`] crate to send log messages to the systemd journal:
//!
//! ```edition2018
//! use log::{info, warn, error, LevelFilter};
//!
//! # fn main(){
//! systemd_journal_logger::init().unwrap();
//! log::set_max_level(LevelFilter::Info);
//!
//! info!("hello log");
//! warn!("warning");
//! error!("oops");
//! # }
//! ```
//!
//! # Journal fields
//!
//! The journald logger always sets the following standard [journal fields][2]
//!
//! - `PRIORITY`: The log level mapped to a priority (see below).
//! - `MESSAGE`: The formatted log message (see [`log::Record::args()`]).
//! - `SYSLOG_IDENTIFIER`: The short name of the running process, i.e. the file name of [`std::env::current_exe()`] if successful.
//! - `SYSLOG_PID`: The PID of the running process (see [`std::process::id()`]).
//! - `CODE_FILE`: The filename the log message originates from (see [`log::Record::file()`], only if present).
//! - `CODE_LINE`: The line number the log message originates from (see [`log::Record::line()`], only if present).
//!
//! Additionally it also adds the following non-standard fields:
//!
//! - `TARGET`: The target of the log record (see [`log::Record::target()`]).
//! - `RUST_MODULE_PATH`: The module path of the log record (see [`log::Record::module_path()`], only if present).
//!
//! [2]: https://www.freedesktop.org/software/systemd/man/systemd.journal-fields.html
//!
//! # Log levels and priorities
//!
//! The journal logger maps [`log::Level`] to journal priorities as follows:
//!
//! - [`Level::Error`] → [`libsystemd::logging::Priority::Error`]
//! - [`Level::Warn`] → [`libsystemd::logging::Priority::Warning`]
//! - [`Level::Info`] → [`libsystemd::logging::Priority::Info`]
//! - [`Level::Debug`] → [`libsystemd::logging::Priority::Debug`]
//! - [`Level::Trace`] → [`libsystemd::logging::Priority::Debug`]

use std::borrow::Cow;

use libsystemd::logging::{journal_send, Priority};
use log::{Level, Log, Metadata, Record, SetLoggerError};

/// A systemd journal logger.
pub struct JournalLog;

/// Convert a a log level to a journal priority.
fn level_to_priority(level: Level) -> Priority {
    match level {
        Level::Error => Priority::Error,
        Level::Warn => Priority::Warning,
        Level::Info => Priority::Info,
        Level::Debug => Priority::Debug,
        Level::Trace => Priority::Debug,
    }
}

/// Collect the standard journal fields from `record`.
///
/// Sets
///
/// - the standard `SYSLOG_IDENTIFIER` and `SYSLOG_PID` fields to the short name (if it exists)
///   and the PID of the current process.
/// - the sta`  ndard `CODE_FILE` and `CODE_LINE` fields to the file and line of `record`,
/// - the custom `TARGET` field to `record.target`, and
/// - the custom `RUST_MODULE_PATH` to the module path of `record`.
fn standard_fields<'a>(record: &'a Record) -> Vec<(&'static str, Cow<'a, str>)> {
    let mut fields = Vec::with_capacity(6);

    if let Some(short_name) = std::env::current_exe()
        .ok()
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
    {
        fields.push(("SYSLOG_IDENTIFIER", short_name.into()));
    }
    fields.push(("SYSLOG_PID", std::process::id().to_string().into()));

    if let Some(file) = record.file() {
        fields.push(("CODE_FILE", file.into()));
    }
    if let Some(line) = record.line().map(|l| l.to_string()) {
        fields.push(("CODE_LINE", line.into()));
    }
    // Non-standard fields
    fields.push(("TARGET", record.target().into()));
    if let Some(module) = record.module_path() {
        fields.push(("RUST_MODULE_PATH", module.into()))
    }
    fields
}

impl Log for JournalLog {
    /// Whether this logger is enabled.
    ///
    /// Always returns `true`.
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    /// Send the given `record` to the systemd journal.
    fn log(&self, record: &Record) {
        journal_send(
            level_to_priority(record.level()),
            &format!("{}", record.args()),
            standard_fields(record).into_iter(),
        )
        // TODO: Figure out how to handle journal write failures
        .ok();
    }

    /// Flush log records.
    ///
    /// A no-op for journal logging.
    fn flush(&self) {}
}

/// The static instance of systemd journal logger.
pub static LOG: JournalLog = JournalLog;

/// Initialize journal logging.
///
/// Set [`LOG`] as logger with [`log::set_logger`].
///
/// This function can only be called once during the lifetime of a program.
///
/// # Default level
///
/// This function does not configure any default level for
/// [`log`]; you need to configure the maximum level explicitly with
/// [`log::set_max_level`], as [`log`] normally defaults to
/// [`log::LevelFilter::Off`] which disables all logging.
///
/// # Errors
///
/// This function returns an error if any logger was previously registered.
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOG)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use libsystemd::logging::Priority;
    use log::{Level, Record};
    use pretty_assertions::assert_eq;

    #[test]
    fn level_to_priority() {
        assert!(matches!(
            crate::level_to_priority(Level::Error),
            Priority::Error
        ));
        assert!(matches!(
            crate::level_to_priority(Level::Warn),
            Priority::Warning
        ));
        assert!(matches!(
            crate::level_to_priority(Level::Info),
            Priority::Info
        ));
        assert!(matches!(
            crate::level_to_priority(Level::Debug),
            Priority::Debug
        ));
        assert!(matches!(
            crate::level_to_priority(Level::Trace),
            Priority::Debug
        ));
    }

    #[test]
    fn standard_fields() {
        let record = Record::builder()
            .level(Level::Warn)
            .line(Some(10))
            .file(Some("foo.rs"))
            .target("testlog")
            .module_path(Some(module_path!()))
            .build();
        let fields = super::standard_fields(&record);

        assert_eq!(fields[0].0, "SYSLOG_IDENTIFIER");
        assert!(
            (&fields[0].1).contains("systemd_journal_logger"),
            "SYSLOG_IDENTIFIER={}",
            fields[0].1
        );

        assert_eq!(fields[1].0, "SYSLOG_PID");
        assert_eq!((&fields[1].1).as_ref(), std::process::id().to_string());

        assert_eq!(
            fields[2..],
            vec![
                ("CODE_FILE", Cow::Borrowed("foo.rs")),
                ("CODE_LINE", Cow::Borrowed("10")),
                ("TARGET", Cow::Borrowed("testlog")),
                (
                    "RUST_MODULE_PATH",
                    Cow::Borrowed("systemd_journal_logger::tests")
                )
            ]
        )
    }
}
