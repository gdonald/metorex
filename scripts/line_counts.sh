#!/usr/bin/env bash
# Count lines in Rust source files and output sorted by line count (descending).
# Used for identifying files that may need refactoring.

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC_DIR="$SCRIPT_DIR/../src"

# Check if src directory exists
if [ ! -d "$SRC_DIR" ]; then
  echo "Error: $SRC_DIR does not exist"
  exit 1
fi

# Find all .rs files and count lines
declare -a files
declare -a counts

while IFS= read -r file; do
  if [ -f "$file" ]; then
    line_count=$(wc -l < "$file")
    relative_path="${file#$SCRIPT_DIR/../}"
    files+=("$relative_path")
    counts+=("$line_count")
  fi
done < <(find "$SRC_DIR" -type f -name "*.rs")

# Check if any files were found
if [ ${#files[@]} -eq 0 ]; then
  echo "No Rust files found in $SRC_DIR"
  exit 0
fi

# Sort arrays by line count (descending)
# Create array of indices sorted by count
indices=()
for i in "${!counts[@]}"; do
  indices+=("$i")
done

# Bubble sort indices based on counts (descending)
for ((i = 0; i < ${#indices[@]}; i++)); do
  for ((j = i + 1; j < ${#indices[@]}; j++)); do
    if [ "${counts[${indices[$i]}]}" -lt "${counts[${indices[$j]}]}" ]; then
      # Swap indices
      tmp="${indices[$i]}"
      indices[$i]="${indices[$j]}"
      indices[$j]="$tmp"
    fi
  done
done

# Output results
printf "%-60s %10s\n" "File" "Lines"
printf "%s\n" "----------------------------------------------------------------------"

total_lines=0
for idx in "${indices[@]}"; do
  file="${files[$idx]}"
  count="${counts[$idx]}"
  total_lines=$((total_lines + count))

  # Only show files with 100+ lines
  if [ "$count" -ge 100 ]; then
    printf "%-60s %10d\n" "$file" "$count"
  fi
done

printf "%s\n" "----------------------------------------------------------------------"
printf "%-60s %10d\n" "Total" "$total_lines"
printf "\nTotal files: %d\n" "${#files[@]}"
