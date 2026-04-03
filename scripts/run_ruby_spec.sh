#!/usr/bin/env bash

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
METOREX="$REPO_ROOT/target/debug/metorex"
MSPEC="$REPO_ROOT/ruby/mspec/bin/mspec"
SPEC_DIR="$REPO_ROOT/ruby/spec"

if [ ! -f "$METOREX" ]; then
  echo "Building metorex..."
  cargo build --manifest-path "$REPO_ROOT/Cargo.toml"
fi

# Passing specs - add new lines as specs reach a passing state
"$MSPEC" -t "$METOREX" "$SPEC_DIR/core/true"
"$MSPEC" -t "$METOREX" "$SPEC_DIR/core/nil"
