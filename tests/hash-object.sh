#!/bin/bash
. $(dirname $0)/common.inc
cd $(setup_new)

content="Hello, world!"

hash=$(echo $content | $GIT hash-object --stdin | grep -E "^[0-9a-z]{40}$")
! test -f ".git/objects/${hash:0:2}/${hash:2}" || false

hash=$(echo $content | $GIT hash-object --stdin -w | grep -E "^[0-9a-z]{40}$")
test -f ".git/objects/${hash:0:2}/${hash:2}"

hash=$(echo "README.md" | $GIT hash-object --stdin-paths -w | grep -E "^[0-9a-z]{40}$")
test -f ".git/objects/${hash:0:2}/${hash:2}"