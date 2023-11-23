#!/bin/bash

cd "$(dirname "$0")"/../
. tests/common.sh

exit_code=0
while read -r file; do
  if [[ "$file" == "tests/all.sh" ]]; then
    continue
  fi
  echo "Running $file"
  bash "$file"
  status=$?

  if [[ $status -ne 0 ]]; then
    exit_code=$status
    echo_red FAIL
  fi
  echo
done < <(find tests -name '*.sh' -type f -perm -u+x)

if [[ $exit_code -eq 0 ]]; then
  echo_green "All tests passed"
else
  echo_red "Some tests failed"
fi
exit $exit_code