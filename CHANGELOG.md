# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Run [`cargo-release`][cr] to publish a release.

[cr]: https://github.com/crate-ci/cargo-release/

## [Unreleased]

### Added
- Add `JournalLog::default`, and new setters for extra fields and the syslog identifier (see [GH-17]).
- Add `with_syslog_identifier` to override the syslog identifier (see [GH-16] and [GH-17]).

### Changed
- Cache the syslog identifier instead of computing it for every log message (see [GH-16] and [GH-17]).

### Removed
- Static `LOG` instance.  Always create your own `JournalLog` instance now (see [GH-17]).
- `init` and `init_with_extra_fields`.  Create your own `JournalLog` instance now, and call `install` (see [GH-17]).

[GH-16]: https://github.com/swsnr/systemd-journal-logger.rs/issues/16
[GH-17]: https://github.com/swsnr/systemd-journal-logger.rs/pull/17

## [0.7.0] – 2022-12-23

### Changed
- Bump `libsystemd` dependency to `0.6.0`

## [0.6.0] – 2022-12-02

### Changed
- Bump MRSV to 1.56.0 (see [GH-11]).
- Update Github URL to <https://github.com/swsnr/systemd-journal-logger.rs>.

[GH-11]: https://github.com/swsnr/systemd-journal-logger.rs/pull/11

## [0.5.1] – 2022-10-15

### Changed
- Move repository, issues, etc. back to <https://github.com/swsnr/systemd-journal-logger.rs>.

## [0.5.0] – 2022-01-28

### Changed
- Move repository, issues, etc. to <https://codeberg.org/flausch/systemd-journal-logger.rs>.
- Update to libsystemd 0.5.0.

## [0.4.1] – 2021-11-20

### Changed
- Bump libsystemd to `0.4.1` to support socket reuse and fix memfd leak.

## [0.4.0] – 2021-10-28

### Changed
- Update to `libsystemd` 0.4.0, and reexport `connected_to_journal()` from that crate.

### Removed
- Dependencies on `libc` and `nix`.

## [0.3.1] – 2021-10-06

### Fixed
- Compile on arm7 targets (see [GH-7]).

[GH-7]: https://github.com/swsnr/systemd-journal-logger.rs/pull/7

## [0.3.0] – 2021-06-07

### Added
- Add `JournalLog::with_extra_fields` and `init_with_extra_fields` to add custom fields to every log entry (see [GH-3]).
- Add new `journal_send` to send an individual log record directly to the systemd journal.
- Add new `connected_to_journal` function to check for a direct connection to the systemd log in order to automatically upgrade to journal logging in systemd services (see [GH-5]).
- Add record key values as custom journal fields (see [GH-1]).

### Changed
- Do not silently ignore journal errors; instead panic if the logger fails to send a message to the systemd journal.
- Use `CODE_MODULE` field for the Rust module path, for compatibility with the `slog-journal` and `systemd` crates.

[GH-1]: https://github.com/swsnr/systemd-journal-logger.rs/pull/1
[GH-3]: https://github.com/swsnr/systemd-journal-logger.rs/pull/3
[GH-5]: https://github.com/swsnr/systemd-journal-logger.rs/pull/5

## [0.2.0] – 2021-06-01

### Fixed

- Multiline messages are no longer lost (see [GH-2]), following an update of [libsystemd] (see [libsystemd GH-70] and [libsystemd GH-72]).

[GH-2]: https://github.com/swsnr/systemd-journal-logger.rs/pull/2
[libsystemd]: https://github.com/lucab/libsystemd-rs
[libsystemd GH-70]: https://github.com/lucab/libsystemd-rs/issues/70
[libsystemd GH-72]: https://github.com/lucab/libsystemd-rs/pull/72

## [0.1.0] – 2021-05-28 (yanked)

Initial release with `systemd_journal_logger::LOG` and `systemd_journal_logger::init`.

Do **not use** this version; it looses multiline messages and has been yanked from crates.io.

[Unreleased]: https://github.com/swsnr/systemd-journal-logger.rs/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/swsnr/systemd-journal-logger.rs/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/swsnr/systemd-journal-logger.rs/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/swsnr/systemd-journal-logger.rs/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/swsnr/systemd-journal-logger.rs/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/swsnr/systemd-journal-logger.rs/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/swsnr/systemd-journal-logger.rs/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/swsnr/systemd-journal-logger.rs/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/swsnr/systemd-journal-logger.rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/swsnr/systemd-journal-logger.rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/swsnr/systemd-journal-logger.rs/releases/tag/v0.1.0
