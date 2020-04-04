#!/bin/bash
set -eu

root="$(cd "$(dirname "$0")/.."; pwd)"

target="$1"; shift
dir="$1"; shift

mkdir -p "$dir"

for ext in dylib so; do
  target_path="${root}/target/${target}/libcore.${ext}"
  if [[ -f "$target_path" ]]; then
    cp -f "$target_path" "${dir}/core.so"
  fi
done
