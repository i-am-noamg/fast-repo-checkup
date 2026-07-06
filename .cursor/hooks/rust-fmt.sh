#!/usr/bin/env bash
# Format Rust sources after Cursor agent file edits (format-on-save does not run for agent writes).
set -euo pipefail

input=$(cat)
file_path=$(echo "$input" | jq -r '.file_path // empty')

if [[ -z "$file_path" || "$file_path" != *.rs ]]; then
  exit 0
fi

if command -v cargo >/dev/null 2>&1; then
  cargo fmt --all
fi

exit 0
