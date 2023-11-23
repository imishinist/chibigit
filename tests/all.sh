#!/bin/bash

set -e
cd "$(dirname "$0")"/../

find tests -name '*.sh' -type f -perm -u+x | while read -r file; do
  if [[ "$file" == "tests/all.sh" ]]; then
    continue
  fi
  echo "Running $file"
  bash "$file"
done