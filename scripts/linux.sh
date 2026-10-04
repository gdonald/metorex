#!/usr/bin/env bash
# Run a command against a Linux build of metorex, in the image CI uses.
#
#   scripts/linux.sh                          # cargo test
#   scripts/linux.sh cargo build
#   scripts/linux.sh cargo clippy --all-targets
#   scripts/linux.sh scripts/run_ruby_spec.sh
#
# The architecture is settled by LINUX_ARCH, which is `arm64` or `amd64` and
# defaults to this machine's own. CI runs on `amd64`, and an architecture that
# is not the host's runs under emulation, which is slower but catches what
# only shows there: the signedness of a C character, the width of a pointer,
# the layout of a struct.
#
#   LINUX_ARCH=amd64 scripts/linux.sh cargo test
#
# A test that hangs on Linux is named by running the suite one test at a time,
# since the name is printed before the test runs and the last line without an
# `ok` after it is the one that stopped:
#
#   scripts/linux.sh cargo test -- --test-threads=1
#
# Each architecture builds into a volume of its own, mounted over target/, so
# it builds to the same path CI does while leaving the host's own build alone.
# The cargo registry is kept in a volume too, so a rebuild does not download
# the dependencies again.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)

case "${LINUX_ARCH:-$(uname -m)}" in
  amd64 | x86_64) ARCH=amd64 ;;
  arm64 | aarch64) ARCH=arm64 ;;
  *)
    echo "scripts/linux.sh: LINUX_ARCH must be amd64 or arm64" >&2
    exit 2
    ;;
esac

IMAGE="metorex-linux-$ARCH"

# An architecture that is not this machine's runs under emulation, which the
# spec runner is told so it can leave out what only a native run can show.
case "$(uname -m)" in
  x86_64 | amd64) HOST_ARCH=amd64 ;;
  *) HOST_ARCH=arm64 ;;
esac
EMULATED=0
if [ "$ARCH" != "$HOST_ARCH" ]; then
  EMULATED=1
fi

# The build context holds the Dockerfile alone, since the repository itself is
# mounted rather than copied.
docker build \
  --platform "linux/$ARCH" \
  --tag "$IMAGE" \
  --build-arg "UID=$(id -u)" \
  --build-arg "GID=$(id -g)" \
  "$ROOT/docker" >&2

if [ "$#" -eq 0 ]; then
  set -- cargo test
fi

terminal=()
if [ -t 0 ] && [ -t 1 ]; then
  terminal=(--tty --interactive)
fi

# The image carries an account with the host's own ids, which the container
# runs as. A test that asks what a file refuses reads differently for root.
# Coverage watches the test process as it runs, which a container refuses by
# default, so the two settings that let it are given here. The C extensions
# the spec suite compiles are kept per architecture, as the build is, since
# one built for the other architecture cannot be loaded.
exec docker run --rm "${terminal[@]}" \
  --platform "linux/$ARCH" \
  --cap-add=SYS_PTRACE \
  --security-opt seccomp=unconfined \
  --volume "$ROOT:/work" \
  --volume "metorex-linux-target-$ARCH:/work/target" \
  --volume "metorex-linux-spec-ext-$ARCH:/work/ruby/spec/ext" \
  --volume "metorex-cargo-registry-$ARCH:/usr/local/cargo/registry" \
  --workdir /work \
  --env "METOREX_EMULATED=$EMULATED" \
  "$IMAGE" "$@"
