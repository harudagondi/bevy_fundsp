set windows-powershell := true

ci: check-fmt clippy test-all

test-all:
    cargo test
    -cargo test --no-default-features --features bevy_audio
    -cargo test --no-default-features --features kira
    -cargo test --no-default-features --features oddio

test feature:
    cargo test --no-default-features --features 

check-fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy