#!/usr/bin/env bash
# check-file-size.sh — enforce the CLAUDE.md "no god file" cap (≤600 LOC), from chetto1983/ModbusTool (itself from Aura).
# Covers Rust, TypeScript, Svelte and Structured Text sources (tests included); node_modules, build output
# and declaration files are exempt.
#
# Usage:
#   bash scripts/check-file-size.sh               # whole tree (CI)
#   bash scripts/check-file-size.sh a.rs b.st     # only the given files (lefthook {staged_files})

set -euo pipefail

CAP=600

select_targets() {
  grep -E '\.(rs|ts|svelte|mjs|st)$' \
    | grep -v -E '(^|/)(node_modules|dist|target)/' \
    | grep -v -E '\.d\.ts$' \
    || true
}

if [ "$#" -gt 0 ]; then
  TARGETS=$(printf '%s\n' "$@" | select_targets)
else
  TARGETS=$(git ls-files | select_targets)
fi

violations=0
checked=0
while IFS= read -r path; do
  [ -n "$path" ] && [ -f "$path" ] || continue
  checked=$((checked + 1))
  lines=$(wc -l < "$path")
  if [ "$lines" -gt "$CAP" ]; then
    printf "OVER CAP: %s (%d LOC > %d)\n" "$path" "$lines" "$CAP"
    violations=$((violations + 1))
  fi
done <<< "$TARGETS"

if [ "$violations" -gt 0 ]; then
  echo "check-file-size: $violations file(s) exceed the ${CAP}-LOC cap. Split by concern (CLAUDE.md)." >&2
  exit 1
fi
echo "check-file-size: $checked file(s) within the ${CAP}-LOC cap."
