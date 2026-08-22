#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "usage: scripts/bump-version.sh <new-version>" >&2
  exit 1
fi

new_version="$1"
root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

sed -i "s/^  \"version\": \".*\"/  \"version\": \"$new_version\"/" "$root_dir/package.json"
sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$root_dir/src-tauri/Cargo.toml"

echo "Bumped to $new_version. Review the diff, then:"
echo "  git add -A && git commit -m \"chore: bump version to $new_version\""
echo "  git tag v$new_version && git push origin master v$new_version"
