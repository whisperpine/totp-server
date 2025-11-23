# list all available subcommands
_default:
  @just --list

# run totp-server with "debug" log level
run:
  RUST_LOG="totp_server=debug" \
  cargo run

# profile totp-server by cargo flamegraph
flame:
  cargo flamegraph --dev

# find vulnerabilities and misconfigurations by trivy
trivy:
  trivy fs --skip-dirs "./target" .
  trivy config .

# run tests and report code coverage in html format
cov:
  cargo llvm-cov nextest --html --open

# compile AWS Lambda functions according to CargoLambda.toml
build:
  cargo lambda build --release

# cargo nextest the given test case(s) and output logs
t CASE:
  cargo nextest run {{CASE}} --no-capture

# boot the dev server locally that emulates AWS Lambda
watch:
  RUST_LOG="cargo_lambda=info,totp_server=debug" \
  cargo lambda watch
