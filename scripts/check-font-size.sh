#!/usr/bin/env bash
# A1 of spec 2026-11-26-ux-consolidation-design.md: every font size in the app's styles is one of app.css's --text-*
# tokens (a 13 px body, nothing below 11 px). Fails on a font-size, or a font shorthand, with any other size.
set -euo pipefail
cd "$(dirname "$0")/../app/src"
found=$(grep -rnE --include=*.svelte --include=*.css '(font-size|(^|[^-])font):' . |
  grep -v '/paraglide/' |
  grep -vE 'font-size: *var\(--text-(xs|sm|md|xl)\);' |
  grep -vE '(^|[^-])font: *inherit;' || true)
if [ -n "$found" ]; then
  echo "Font sizes outside the --text-* tokens (app/src/app.css):" >&2
  echo "$found" >&2
  exit 1
fi
echo "check-font-size: every font size is a --text-* token."
