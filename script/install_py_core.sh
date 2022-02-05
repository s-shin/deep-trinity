#!/bin/bash
set -eu

: "${LIB_NAME:=deep_trinity}"

script_dir="$(cd "$(dirname "$0")" && pwd)"
project_root_dir="$(cd "${script_dir}/.." && pwd)"

usage() {
  cat <<EOT
Usage: $0 [options] <outdir>

Options:
  -h, --help
  -p, --target_profile <profile> (default: debug)
EOT
}

abort() { >&2 echo "ERROR: $*"; exit 1; }

main() {
  local target_profile=debug
  local nargs=0
  local outdir=''
  while (($# > 0)); do
    case "$1" in
      -h | --help ) usage; exit 1;;
      -p | --target_profile ) target_profile="$2"; shift;;
      -* ) abort "Unknown option: $1";;
      * )
        case $((++nargs)) in
          1 ) outdir="$1";;
          * ) abort 'Too many arguments';;
        esac
    esac
    shift
  done

  mkdir -p "$outdir"

  for ext in dylib so; do
    local lib="${project_root_dir}/target/${target_profile}/lib${LIB_NAME}.${ext}"
    if [[ -f "$lib" ]]; then
      # cf. https://pyo3.rs/v0.15.1/building_and_distribution.html#manual-builds
      cp -f "$lib" "${outdir}/${LIB_NAME}.so"
      exit 0
    fi
  done

  abort 'No library file found.'
}

main "$@"
