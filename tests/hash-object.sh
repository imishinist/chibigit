#!/bin/bash

set -e
. "$(dirname "$0")"/common.sh

content="Hello, world!"
diff \
  <(echo $content | $GIT hash-object --stdin) \
  <(echo $content | $CGIT hash-object --stdin)
echo "hash-object --stdin => OK"