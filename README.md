# systemd-journal-logger

A [log] logger for the [systemd journal][1].

[log]: https://docs.rs/log
[1]: https://www.freedesktop.org/software/systemd/man/systemd-journald.service.html

## Usage

```toml
[dependencies]
log = "^0.4"
systemd-journal-logger = "^0.1"
```

Then initialize the logger at the start of `main`:

```rust
use log::{info, LevelFilter};

fn main() {
    systemd_journal_logger::init();
    log::set_max_level(LevelFilter::Info);
    
    info!("Hello systemd journal");
}
```