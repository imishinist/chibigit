#!/bin/bash
. $(dirname $0)/common.inc

cd $(setup_new)

# git ls-files --stage output
$GIT ls-files | grep -E "^[0-9]{6} [0-9a-z]{40} 0\t.*$" >/dev/null