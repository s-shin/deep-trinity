#!/bin/bash
set -eu

cd "$(dirname "$0")/.."

target="$1"; shift
dir="$1"; shift

mkdir -p "$dir"

for ext in dylib so; do
  target_path="target/${target}/libdetris.${ext}"
  if [[ -f "$target_path" ]]; then
    cp -f "$target_path" "${dir}/detris.so"
  fi
done
