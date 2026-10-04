#!/bin/sh

# Purpose: check rust codebase by multiple cargo subcommands
# Usage: sh path/to/cargo-validation.sh
# Dependencies: cargo, cargo-nextest
# Date: 2026-06-20
# Author: Yusong

set -e

exit_code=0

run_command() {
  cmd="$*"
  if ! eval "$cmd"; then
    printf "\n### Info: Above are issues found by '%s'.\n\n" "$cmd"
    exit_code=1
  fi
}

run_command cargo fmt --check

run_command cargo clippy --release --all-features -- -D warnings
# run_command cargo clippy --examples --release --all-features -- -D warnings
run_command cargo clippy --tests --all-features -- -D warnings

run_command cargo nextest run --all-features

RUSTDOCFLAGS="-D warnings" run_command cargo doc --no-deps --all-features
# RUSTDOCFLAGS="-D warnings" run_command cargo doc --no-deps --examples
run_command cargo test --doc --all-features

if [ $exit_code -ne 0 ]; then
  exit "$exit_code"
fi
