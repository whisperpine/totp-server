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

# run tests and report code coverage in html format
cov:
    cargo llvm-cov nextest --html --open
