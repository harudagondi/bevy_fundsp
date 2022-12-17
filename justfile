set windows-powershell := true

default: ci

ci: check-fmt clippy test-all

test-all:
    cargo test
    cargo test --no-default-features --features bevy_audio
    cargo test --no-default-features --features kira
    cargo test --no-default-features --features oddio

check-fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy
    cargo clippy --no-default-features --features bevy_audio
    cargo clippy --no-default-features --features kira
    cargo clippy --no-default-features --features oddio

example feature example:
    cargo run --example {{example}} --no-default-features --features {{feature}} --release 