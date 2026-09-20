#!/usr/bin/env bash
# Everything CI runs, on this machine and on Linux, on both architectures.
#
#   ./test.sh              # all of it
#   ./test.sh mac          # this machine alone
#   ./test.sh linux        # both Linux architectures
#   ./test.sh linux-amd64  # the one CI runs on
#   ./test.sh linux-arm64
#
# Linux needs Docker running. What it runs is in docker/Dockerfile, and
# scripts/linux.sh is what puts a command in front of it. The architecture
# that is not this machine's runs under emulation, which is slower but catches
# what only shows there: the signedness of a C character, the width of a
# pointer, the layout of a struct.
set -uo pipefail

ROOT=$(cd "$(dirname "$0")" && pwd)
cd "$ROOT"

WHERE=${1:-both}

# Run one step, and stop the whole run where it fails, so what went wrong is
# the last thing on the screen rather than something to scroll back for.
step() {
  local label=$1
  shift
  printf '\n\033[1m== %s\033[0m\n' "$label"
  if "$@"; then
    return 0
  fi
  printf '\n\033[1mFailed: %s\033[0m\n' "$label"
  exit 1
}

run_mac() {
  step "mac: fmt" cargo fmt -- --check
  step "mac: clippy" cargo clippy --all-targets -- -D warnings
  step "mac: build" cargo build
  step "mac: tests" cargo test
  step "mac: ruby spec" scripts/run_ruby_spec.sh
  step "mac: tests are in tests/" scripts/misplaced_tests.sh
}

# This machine's own architecture, which is the only one coverage can measure:
# watching a process as it runs needs the real processor, not an emulated one.
host_arch() {
  case "$(uname -m)" in
    x86_64 | amd64) echo amd64 ;;
    *) echo arm64 ;;
  esac
}

# The same steps CI runs, in the image it runs them in. Coverage is included
# because it compiles the tests its own way, and has failed where an ordinary
# build passed.
run_linux() {
  local arch=$1
  if ! docker info >/dev/null 2>&1; then
    printf '\n\033[1mFailed: linux, since Docker is not running\033[0m\n'
    exit 1
  fi
  export LINUX_ARCH="$arch"
  step "linux/$arch: fmt" scripts/linux.sh cargo fmt -- --check
  step "linux/$arch: clippy" scripts/linux.sh cargo clippy --all-targets -- -D warnings
  step "linux/$arch: build" scripts/linux.sh cargo build --verbose
  step "linux/$arch: tests" scripts/linux.sh cargo test --verbose
  step "linux/$arch: ruby spec" scripts/linux.sh bash -c 'cargo build && scripts/run_ruby_spec.sh'
  if [ "$arch" = "$(host_arch)" ]; then
    step "linux/$arch: coverage" scripts/linux.sh \
      cargo tarpaulin --all-features --workspace --timeout 120 --out xml
  else
    # Coverage cannot run under emulation, so what is checked here is the
    # part that broke CI before: that the tests build the way it builds them.
    step "linux/$arch: coverage builds" scripts/linux.sh \
      env RUSTFLAGS="--cfg=tarpaulin -Clink-dead-code" cargo test --no-run
  fi
}

case "$WHERE" in
  mac) run_mac ;;
  linux-amd64) run_linux amd64 ;;
  linux-arm64) run_linux arm64 ;;
  linux)
    run_linux amd64
    run_linux arm64
    ;;
  both)
    run_mac
    run_linux amd64
    run_linux arm64
    ;;
  *)
    echo "usage: $0 [mac|linux|linux-amd64|linux-arm64|both]" >&2
    exit 2
    ;;
esac

printf '\n\033[1mAll steps passed.\033[0m\n'
