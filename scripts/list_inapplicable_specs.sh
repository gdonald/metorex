#!/usr/bin/env bash
#
# Every spec file that runs no examples at all on this platform at the Ruby
# version metorex reports. A file lands here when its examples are guarded
# away: the behavior was removed in a later Ruby, it belongs to another
# platform, or it needs a feature this build does not have. None of them can
# pass, so none of them counts as remaining work.

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
METOREX="$REPO_ROOT/target/debug/metorex"
MSPEC="$REPO_ROOT/ruby/mspec/bin/mspec"
SPEC_DIR="$REPO_ROOT/ruby/spec"
JOBS="${JOBS:-$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)}"

export RUBY_EXE="$METOREX"
export METOREX MSPEC SPEC_DIR

WORK_DIR=$(mktemp -d -t metorex_inapplicable.XXXXXX)
trap 'rm -rf "$WORK_DIR"' EXIT

CHECK="$WORK_DIR/check.sh"
cat > "$CHECK" <<'CHECK_EOF'
#!/usr/bin/env bash
spec=$1
export SPEC_TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$SPEC_TEMP_DIR"' EXIT
out=$(cd "$SPEC_TEMP_DIR" && "$MSPEC" -t "$METOREX" "$SPEC_DIR/$spec" 2>&1 | tr -d '\000')
# A file whose examples all ran prints a line of progress marks. One whose
# examples were every one of them guarded away prints none.
if ! printf '%s' "$out" | grep -qE '^[.EF*]+$'; then
  if printf '%s' "$out" | grep -qE '^1 files?, 0 examples, 0 expectations, 0 failures, 0 errors'; then
    echo "$spec"
  fi
fi
CHECK_EOF
chmod +x "$CHECK"

cd "$SPEC_DIR"
find core language library command_line security -name '*_spec.rb' \
  | sort \
  | xargs -n 1 -P "$JOBS" "$CHECK" \
  | sort
