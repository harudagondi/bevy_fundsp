set windows-powershell := true

check:
    cargo check
    cargo check --no-default-features --features bevy_audio
    cargo check --no-default-features --features kira
    # cargo check --no-default-features --features oddio

test:
    cargo test
    cargo test --no-default-features --features bevy_audio
    cargo test --no-default-features --features kira
    # cargo test --no-default-features --features oddio

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy

ci:
    just fmt
    just clippy
    just check
    just test