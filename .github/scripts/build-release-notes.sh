#!/usr/bin/env bash
set -euo pipefail

release_tag="${1:?release tag is required}"
previous_tag="$(git describe --tags --abbrev=0 "${release_tag}^" 2>/dev/null || true)"
range="${previous_tag:+$previous_tag..}${release_tag}"
repo_url="https://github.com/${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"

{
  echo "## What's new"
  echo

  count=0
  while IFS=$'\t' read -r hash subject; do
    # Conventional commits are the preferred source of release notes. Until the
    # existing history is migrated, keep useful legacy subjects and discard
    # maintenance/refactor noise by pattern.
    legacy_prefix='^(chore|docs|test|ci|build|refactor|style|revert)(\([^)]*\))?:'
    if [[ "$subject" =~ $legacy_prefix ]]; then
      continue
    fi
    if [[ "$subject" =~ ^(Split|Extract|Merge|Bump|Sync|Import)[[:space:]] ]] ||
       [[ "$subject" =~ ^(Update|Refresh)[[:space:]].*(Cargo|dependency|dependencies|lockfile|CI|Actions|workflow) ]] ||
       [[ "$subject" =~ ^Add[[:space:]].*(diagnostic|logging|logs|temporary) ]]; then
      continue
    fi

    clean="$(sed -E 's/^(feat|fix|perf|improve|improvement)(\([^)]*\))?:[[:space:]]*//; s/^[[:space:]]+//; s/[[:space:]]+$//' <<< "$subject")"
    [[ -n "$clean" ]] || continue
    echo "- $clean ([$hash]($repo_url/commit/$hash))"
    count=$((count + 1))
    (( count >= 12 )) && break
  done < <(git log "$range" --no-merges --pretty=format:"%h%x09%s")

  if (( count == 0 )); then
    echo "- Maintenance and reliability improvements"
  fi
} > notes.md
