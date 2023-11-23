#!/bin/bash

set -e
. "$(dirname "$0")"/common.sh
DIR=$(pwd)

content="Hello, world!"

echo -n "hash-object --stdin => "
diff \
  <(echo $content | $GIT hash-object --stdin) \
  <(echo $content | $CGIT hash-object --stdin)
echo_green OK

echo -n "hash-object --stdin -w => "
diff \
  <(cd $(mktemp -d) && \
      $GIT init >/dev/null && \
      echo $content | $GIT hash-object --stdin -w | xargs $GIT cat-file -p) \
  <(cd $(mktemp -d) && \
      mkdir -p target/debug && \
      cp $DIR/target/debug/chibigit target/debug/chibigit && \
      $CGIT init >/dev/null && \
      echo $content | $CGIT hash-object --stdin -w | xargs $CGIT cat-file -p)
echo_green OK