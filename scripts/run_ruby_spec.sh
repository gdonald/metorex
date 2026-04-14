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

# 4.1 - 4.3: Primitives and Singletons
"$MSPEC" -t "$METOREX" "$SPEC_DIR/core/true"
"$MSPEC" -t "$METOREX" "$SPEC_DIR/core/nil"
"$MSPEC" -t "$METOREX" "$SPEC_DIR/core/false"

# 4.4 - 4.9: Core Classes and Modules
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/comparable"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/main"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/class"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/module"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/kernel"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/basicobject"

# 4.10 - 4.13: Exceptions and Signals
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/exception"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/systemexit"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/signal"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/warning"

# 4.14 - 4.19: Numeric Types
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/integer"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/float"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/numeric"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/complex"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/rational"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/math"

# 4.20 - 4.25: Collections
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/array"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/hash"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/set"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/range"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/struct"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/data"

# 4.26 - 4.30: Strings and Patterns
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/string"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/symbol"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/regexp"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/encoding"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/matchdata"

# 4.31 - 4.34: Callables and Introspection
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/proc"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/method"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/unboundmethod"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/binding"

# 4.35 - 4.39: IO and Filesystem
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/io"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/file"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/dir"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/filetest"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/env"

# 4.40 - 4.46: Concurrency
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/thread"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/fiber"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/mutex"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/conditionvariable"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/queue"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/sizedqueue"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/threadgroup"

# 4.47 - 4.55: Other
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/gc"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/objectspace"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/random"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/time"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/process"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/marshal"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/tracepoint"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/refinement"
# "$MSPEC" -t "$METOREX" "$SPEC_DIR/core/builtin_constants"
