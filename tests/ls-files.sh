#!/bin/bash

set -e
. "$(dirname "$0")"/common.sh

diff \
  <($GIT ls-files --stage) \
  <($CGIT ls-files)
echo "ls-files => OK"