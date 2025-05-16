default:
    just --list

vet:
    cargo vet --locked

test-all: vet
    cargo +stable deny --all-features --locked check
    cargo +stable fmt -- --check
    cargo +stable clippy --all-targets --locked
    # Stable build, test, and docs
    cargo +stable build --locked
    cargo +stable test --locked
    cargo +stable doc
    # MSRV
    cargo +1.68 build --locked --all-targets
    # semver checks
    cargo semver-checks

release *ARGS: test-all
    cargo release {{ARGS}}
