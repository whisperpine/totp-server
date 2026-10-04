# list all available subcommands
_default:
  @just --list

# run totp-server with "debug" log level
[group("misc")]
run:
  RUST_LOG="totp_server=debug" \
  cargo run

# profile totp-server by cargo flamegraph
[group("misc")]
flame:
  cargo flamegraph --dev

# find vulnerabilities and misconfigurations by trivy
[group("misc")]
trivy:
  trivy fs --skip-dirs "./target" .
  trivy config .

# compile AWS Lambda functions according to CargoLambda.toml
[group("lambda")]
build:
  cargo lambda build --release

# boot the dev server locally that emulates AWS Lambda
[group("lambda")]
watch:
  RUST_LOG="cargo_lambda=info,totp_server=debug" \
  cargo lambda watch

# cargo nextest the given test case(s) and output logs
[group("test")]
t CASE:
  cargo nextest run {{CASE}} --no-capture

# run tests and report code coverage in html format
[group("test")]
cov:
  cargo llvm-cov nextest --html --open

# run rust codebase by multiple cargo subcommands
[group("test")]
validate:
  sh ./scripts/cargo-validation.sh
