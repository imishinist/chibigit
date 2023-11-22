#!/bin/bash

set -e

cargo build
GIT="git"
CGIT="./target/debug/chibigit"

export RUST_BACKTRACE=1

rev=$(git rev-parse HEAD)
diff \
  <($GIT cat-file -p $rev) \
  <($CGIT cat-file -p $rev)
echo "cat-file -p #{commit hash} => OK"

diff \
  <($GIT cat-file -p $($GIT cat-file -p $rev | grep tree | awk '{print $2}')) \
  <($CGIT cat-file -p $($CGIT cat-file -p $rev | grep tree | awk '{print $2}'))
echo "cat-file -p #{tree hash} => OK"

diff \
  <($GIT cat-file -p $($GIT cat-file -p $($GIT cat-file -p $rev | grep tree | awk '{print $2}') | grep blob | head -1 | awk '{print $3}')) \
  <($CGIT cat-file -p $($CGIT cat-file -p $($CGIT cat-file -p $rev | grep tree | awk '{print $2}') | grep blob | head -1 | awk '{print $3}'))
echo "cat-file -p #{blob hash} => OK"