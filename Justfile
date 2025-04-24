# run totp-server with "debug" log level.
run:
    RUST_LOG="totp_server=debug" \
    cargo run

# build the docker image for the local machine's platform.
build:
    docker build -t totp-server .

# build multi-platform docker images (linux/amd64,linux/arm64).
buildp:
    docker build --platform linux/amd64,linux/arm64 -t totp-server .

# find vulnerabilities and misconfigurations by trivy.
trivy:
    trivy fs .
    trivy config .
