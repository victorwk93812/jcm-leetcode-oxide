#!/usr/bin/env bash
# Scaffold a new problem module.
#
#   scripts/new.sh <number> <slug>
#   scripts/new.sh 206 reverse-linked-list
#
# Creates src/problems/p0206_reverse_linked_list.rs from templates/problem.rs
# and registers it in src/problems/mod.rs (kept sorted).
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "usage: $0 <number> <slug>" >&2
    exit 1
fi

number="$1"
slug="$2"

if ! [[ "$number" =~ ^[0-9]+$ ]]; then
    echo "error: number must be an integer, got '$number'" >&2
    exit 1
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
padded="$(printf '%04d' "$number")"
snake="$(echo "$slug" | tr 'A-Z-' 'a-z_' | tr -cd 'a-z0-9_')"
kebab="$(echo "$snake" | tr '_' '-')"
title="$(echo "$snake" | tr '_' ' ' | sed 's/\b\(.\)/\u\1/g')"
modname="p${padded}_${snake}"
file="$root/src/problems/${modname}.rs"
registry="$root/src/problems/mod.rs"

if [[ -e "$file" ]]; then
    echo "error: $file already exists" >&2
    exit 1
fi

sed -e "s/{{NUMBER}}/$number/g" \
    -e "s/{{TITLE}}/$title/g" \
    -e "s/{{SLUG}}/$kebab/g" \
    "$root/templates/problem.rs" > "$file"

echo "pub mod ${modname};" >> "$registry"

# Re-sort the registry: keep the leading comment block, sort the mod lines.
{
    grep -E '^(//|$)' "$registry" || true
    grep -E '^pub mod ' "$registry" | sort -u
} > "$registry.tmp"
mv "$registry.tmp" "$registry"

echo "created $file"
echo "registered ${modname} in src/problems/mod.rs"
echo
echo "run its tests with:  cargo test ${modname}"
