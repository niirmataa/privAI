#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${repo_root}/.claude/skills"
target_home="${CLAUDE_WINDOWS_HOME:-/mnt/c/Users/alicj/.claude}"
target_dir="${target_home}/skills"

if [[ ! -d "${source_dir}" ]]; then
  echo "Source skills directory not found: ${source_dir}" >&2
  exit 1
fi

mkdir -p "${target_dir}"

for skill_dir in "${source_dir}"/privai-*; do
  [[ -d "${skill_dir}" ]] || continue
  skill_name="$(basename "${skill_dir}")"
  rm -rf "${target_dir}/${skill_name}"
  cp -R "${skill_dir}" "${target_dir}/${skill_name}"
  echo "Synced ${skill_name} -> ${target_dir}/${skill_name}"
done

echo "Claude skills synced to ${target_dir}"
