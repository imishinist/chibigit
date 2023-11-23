#!/bin/bash

set -e
. "$(dirname "$0")"/common.sh

echo -n "ls-files => "
diff \
  <($GIT ls-files --stage) \
  <($CGIT ls-files)
echo_green OK