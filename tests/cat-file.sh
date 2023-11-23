#!/bin/bash
. $(dirname $0)/common.inc

cd $(setup_new)

rev=$(git rev-parse HEAD)
$GIT cat-file -p $rev | grep -E "^tree [0-9a-z]{40}$" >/dev/null
$GIT cat-file -p $rev | grep -E "^parent [0-9a-z]{40}$" >/dev/null

tree=$($GIT cat-file -p $rev | grep tree | awk '{print $2}')
$GIT cat-file -p "$tree" | grep -Ei "^[0-9]{6} blob [0-9a-z]{40}\s*.*$" >/dev/null
$GIT cat-file -p $tree | grep -Eq "^[0-9]{6} tree [0-9a-z]{40}\s*.+$" >/dev/null

blob_line=$($GIT cat-file -p "$tree" | grep blob | head -1)
hash=$(echo "$blob_line" | awk '{print $3}')
filename=$(echo "$blob_line" | awk '{print $4}')
diff -q <($GIT cat-file -p "$hash") $filename >/dev/null