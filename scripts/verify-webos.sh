#!/usr/bin/env bash
set -euo pipefail

ipk="${1:?usage: verify-webos.sh <package.ipk> [fw-releases]}"
releases="${2:->=4.0}"

command -v webosbrew-ipk-verify >/dev/null || {
  echo "webosbrew-ipk-verify is not installed (https://github.com/webosbrew/dev-toolbox-cli/releases)" >&2
  exit 3
}

exec webosbrew-ipk-verify --details --fw-releases "$releases" "$ipk"
