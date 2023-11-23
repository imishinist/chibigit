#!/bin/bash

set -e
. "$(dirname "$0")"/common.sh

rev=$(git rev-parse HEAD)
echo -n "cat-file -p #{commit hash} => "
diff \
  <($GIT cat-file -p $rev) \
  <($CGIT cat-file -p $rev)
echo_green "OK"

echo -n "cat-file -p #{tree hash} => "
diff \
  <($GIT cat-file -p $($GIT cat-file -p $rev | grep tree | awk '{print $2}')) \
  <($CGIT cat-file -p $($CGIT cat-file -p $rev | grep tree | awk '{print $2}'))
echo_green "OK"

echo -n "cat-file -p #{blob hash} => "
diff \
  <($GIT cat-file -p $($GIT cat-file -p $($GIT cat-file -p $rev | grep tree | awk '{print $2}') | grep blob | head -1 | awk '{print $3}')) \
  <($CGIT cat-file -p $($CGIT cat-file -p $($CGIT cat-file -p $rev | grep tree | awk '{print $2}') | grep blob | head -1 | awk '{print $3}'))
echo_green "OK"