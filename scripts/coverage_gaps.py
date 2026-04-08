#!/usr/bin/env python3
"""Parse tarpaulin-report.json and show uncovered lines by file."""

import json
import sys

with open('tarpaulin-report.json') as f:
    data = json.load(f)

files_info = []
for f_entry in data['files']:
    path = '/'.join(f_entry['path'])
    traces = f_entry['traces']
    if not traces:
        continue
    uncovered_lines = [t['line'] for t in traces if t['stats']['Line'] == 0]
    total = len(traces)
    unc = len(uncovered_lines)
    if unc > 0:
        short = path.split('metorex/')[-1]
        files_info.append((short, unc, total, uncovered_lines))

files_info.sort(key=lambda x: -x[1])

print(f"Total coverage: {data['coverage']:.1f}%  ({data['covered']}/{data['coverable']})\n")

if '--lines' in sys.argv:
    for short, unc, tot, lines in files_info:
        pct = (tot - unc) / tot * 100
        print(f"{short}: {unc}/{tot} ({pct:.1f}%)")
        print(f"  lines: {lines}\n")
else:
    for short, unc, tot, lines in files_info:
        pct = (tot - unc) / tot * 100
        print(f"{unc:4d} / {tot:4d} ({pct:5.1f}%) {short}")
