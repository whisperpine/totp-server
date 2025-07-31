# run totp-server with "debug" log level
run:
    RUST_LOG="totp_server=debug" \
    cargo run

# build the totp-server container image locally
build:
    docker build -t totp-server .

# find vulnerabilities and misconfigurations by trivy
trivy:
    trivy fs .
    trivy config .

# run tests and report code coverage
cov:
    # Options "--skip-clean" and "--avoid-cfg-tarpaulin" speed up local commands
    # (no interference with cargo test, cargo check), but should not be used in CI.
    cargo tarpaulin --engine llvm --skip-clean --avoid-cfg-tarpaulin --frozen
