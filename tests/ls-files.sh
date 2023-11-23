#!/bin/bash

set -e

cargo build
GIT="git"
CGIT="./target/debug/chibigit"

export RUST_BACKTRACE=1

diff \
  <($GIT ls-files --stage) \
  <($CGIT ls-files)
echo "ls-files => OK"