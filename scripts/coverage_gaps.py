#!/usr/bin/env python3
"""Show uncovered code by file from the last llvm-cov run.

Reads the coverage data `cargo llvm-cov` already collected, through its own
`report` subcommand, so it does not re-run the tests. The vendored Ruby suite
under ruby/ is left out, as it is in the coverage run itself.

  scripts/coverage_gaps.py           # files ranked by uncovered line count
  scripts/coverage_gaps.py --lines   # the uncovered line numbers in each file
"""

import json
import os
import subprocess
import sys

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
IGNORE = ["--ignore-filename-regex", "ruby/"]


def run(*args):
    return subprocess.run(
        ["cargo", "llvm-cov", "report", *args, *IGNORE],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )


if "--lines" in sys.argv:
    # llvm-cov names the uncovered lines itself, which needs one of its file
    # outputs alongside, so the report is written to a cobertura file that is
    # thrown away and the uncovered-line listing is read from stdout.
    result = run("--show-missing-lines", "--cobertura", "--output-path", os.devnull)
    sys.stdout.write(result.stdout)
    sys.exit(0)

report = json.loads(run("--json").stdout)
data = report["data"][0]

files_info = []
for file_entry in data["files"]:
    lines = file_entry["summary"]["lines"]
    total = lines["count"]
    uncovered = total - lines["covered"]
    if uncovered > 0:
        short = os.path.relpath(file_entry["filename"], REPO_ROOT)
        files_info.append((short, uncovered, total))

files_info.sort(key=lambda entry: -entry[1])

totals = data["totals"]["lines"]
print(
    f"Total coverage: {totals['percent']:.1f}%  "
    f"({totals['covered']}/{totals['count']})\n"
)

for short, uncovered, total in files_info:
    pct = (total - uncovered) / total * 100
    print(f"{uncovered:4d} / {total:4d} ({pct:5.1f}%) {short}")
