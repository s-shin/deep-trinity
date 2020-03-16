#!/bin/bash
set -eu

cd "$(dirname "$0")/.."

usage() {
  cat <<EOT
Usage: $0 <template-name> <package-name>
EOT
}

if (($# != 2)); then
  usage
  exit 1
fi

template_name="$1"; shift
package_name="$1"; shift

dst="packages/${package_name}"
if [[ -d "$dst" ]]; then
  echo "ERROR: Destination directory already exists: ${dst}"
  exit 1
fi

cp -R "templates/${template_name}" "$dst"
for file in "${dst}/"*; do
  if [[ -d "$file" ]]; then
    continue
  fi
  perl -i -sple 's/__pkg__/$pkg/g' -- -pkg="${package_name}" "$file"
done

echo 'Done!'
