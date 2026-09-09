#!/usr/bin/env bash
set -euo pipefail

release_tag="${1:?release tag is required}"
repository="${GITHUB_REPOSITORY:-}"
previous_tag="$(git describe --tags --abbrev=0 "${release_tag}^" 2>/dev/null || true)"
range="${previous_tag:+$previous_tag..}${release_tag}"
declare -A seen_subjects=()
declare -A seen_pull_requests=()

is_release_note() {
    local subject="$1"
    local subject_lower
    local conventional_noise='^(build|chore|ci|docs|style|test|refactor|revert)(\([^)]*\))?:'
    local structural_noise='^(split|extract|merge|bump|sync|import)([[:space:][:punct:]]|$)'
    local dependency_noise='^(update|refresh)[[:space:]].*(cargo|dependency|dependencies|lockfile|ci|actions|workflow)'
    local diagnostic_noise='^add[[:space:]].*(diagnostic|logging|logs|temporary)'
    subject_lower="$(printf '%s' "$subject" | tr '[:upper:]' '[:lower:]')"

    [[ "$subject_lower" != *"[skip release notes]"* ]] || return 1
    [[ ! "$subject_lower" =~ $conventional_noise ]] || return 1
    [[ ! "$subject_lower" =~ $structural_noise ]] || return 1
    [[ ! "$subject_lower" =~ $dependency_noise ]] || return 1
    [[ ! "$subject_lower" =~ $diagnostic_noise ]] || return 1
}

resolve_pull_request() {
    local commit="$1"
    [[ -n "$repository" && -n "${GH_TOKEN:-}" ]] || return 0
    command -v gh >/dev/null 2>&1 || return 0

    gh api \
        -H 'Accept: application/vnd.github+json' \
        "repos/${repository}/commits/${commit}/pulls" \
        --jq '
            map(select(.merged_at != null))
            | sort_by(.merged_at)
            | last
            | if . == null then empty else
                [(.number | tostring), (.title | gsub("[\\t\\r\\n]+"; " ")), (.user.login // "")]
                | @tsv
              end
        ' 2>/dev/null || true
}

resolve_username() {
    local commit="$1"
    local author_name="$2"
    local author_email="$3"
    local username=""

    if [[ "$author_email" =~ ^[0-9]+\+([^@]+)@users\.noreply\.github\.com$ ]]; then
        username="${BASH_REMATCH[1]}"
    elif [[ "$author_email" =~ ^([^@]+)@users\.noreply\.github\.com$ ]]; then
        username="${BASH_REMATCH[1]}"
    elif [[ -n "$repository" && -n "${GH_TOKEN:-}" ]] && command -v gh >/dev/null 2>&1; then
        username="$(gh api "repos/${repository}/commits/${commit}" --jq '.author.login // empty' 2>/dev/null || true)"
    fi

    printf '%s' "${username:-$author_name}"
}

{
    echo "## What's Changed"
    echo

    count=0
    separator=$'\x1f'
    while IFS="$separator" read -r full_hash short_hash subject author_name author_email parents; do
        [[ -n "$full_hash" ]] || continue
        is_release_note "$subject" || continue

        pull_data="$(resolve_pull_request "$full_hash")"
        pull_number=""
        pull_title=""
        pull_author=""
        if [[ -n "$pull_data" ]]; then
            IFS=$'\t' read -r pull_number pull_title pull_author <<< "$pull_data"
            [[ -n "${seen_pull_requests[$pull_number]:-}" ]] && continue
            seen_pull_requests["$pull_number"]=1
            subject="$pull_title"
            is_release_note "$subject" || continue
        fi

        normalized_subject="$(printf '%s' "$subject" | tr '[:upper:]' '[:lower:]' | sed -E 's/[[:space:]]+/ /g; s/[[:space:].]+$//')"
        [[ -n "${seen_subjects[$normalized_subject]:-}" ]] && continue
        seen_subjects["$normalized_subject"]=1

        if [[ -n "$pull_author" ]]; then
            username="$pull_author"
        else
            username="$(resolve_username "$full_hash" "$author_name" "$author_email")"
        fi

        printf '%s' "- ${subject}"
        if [[ "$username" =~ ^[A-Za-z0-9-]+$ ]]; then
            if [[ -n "$pull_author" || "$username" != "$author_name" ]]; then
                printf ' by @%s' "$username"
            fi
        fi
        if [[ -n "$pull_number" ]]; then
            printf ' (#%s)' "$pull_number"
        else
            printf ' (`%s`)' "$short_hash"
        fi
        printf '\n'

        count=$((count + 1))
    done < <(
        git log "$range" --first-parent \
            --format="%H${separator}%h${separator}%s${separator}%an${separator}%ae${separator}%P"
    )

    if (( count == 0 )); then
        echo "- Maintenance and reliability improvements"
    fi
} > notes.md
