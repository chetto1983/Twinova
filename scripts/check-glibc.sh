#!/usr/bin/env bash
# Fails when an ELF file needs a glibc above the first argument. The Linux packages must start on the oldest Ubuntu LTS
# still supported, 22.04 with glibc 2.35: v0.4.0's AppImage, built on 24.04, stopped there with "GLIBC_2.39 not found"
# (AppImageHub's test, 2026-11-25), in the app and in the GTK and WebKit libraries it carries.
# Usage: check-glibc.sh <max version> <file>...
set -euo pipefail
max=$1
shift
highest() { objdump -T "$1" 2>/dev/null | grep -o 'GLIBC_[0-9][0-9.]*' | sed 's/^GLIBC_//' | sort -V | tail -1; }
above=0
checked=0
for file in "$@"; do
  file "$file" | grep -q ELF || continue
  checked=$((checked + 1))
  need=$(highest "$file")
  [ -n "$need" ] || continue
  if [ "$(printf '%s\n%s\n' "$need" "$max" | sort -V | tail -1)" != "$max" ]; then
    echo "$file needs glibc $need, above $max" >&2
    above=$((above + 1))
  fi
done
echo "$checked ELF files checked against glibc $max: $above above it"
[ "$checked" -gt 0 ] && [ "$above" -eq 0 ]
