#!/bin/bash

DROP_LIST=$(mktemp)

export GIT="./chibigit"
export RUST_BACKTRACE=1

export GREEN=$(tput setaf 2)
export RED=$(tput setaf 1)
export RESET=$(tput sgr0)

function echo_green() {
  echo "${GREEN}$1${RESET}"
}

function echo_red() {
  echo "${RED}$1${RESET}"
}

function setup_new() {
  git=git
  [[ $# -eq 1 ]] && git=$1

  current_dir=$(pwd)
  dir=$(mktemp -d)
  echo $dir >> $DROP_LIST

  cd $dir
  {
    $git init
    $git commit --allow-empty -m "Initial commit"

    echo "sample content" > README.md
    $git add README.md
    $git commit -m "Add README.md"

    mkdir src
    echo "fn main() { println!(\"Hello, world!\"); }" > src/main.rs
    $git add src/main.rs
    $git commit -m "Add src/main.rs"
  } > /dev/null

  ln -s $current_dir/target/debug/chibigit chibigit
  echo $dir
}

on_error() {
  code=$?
  echo "command failed: $1: $BASH_COMMAND"
  trap - EXIT
  exit $code
}

on_exit() {
  while read -r dir; do
    rm -rf $dir
  done < $DROP_LIST
  echo_green OK
  exit 0
}

trap 'on_error $LINENO' ERR
trap 'on_exit' EXIT

testname=$(basename "$0" .sh)
echo -n "Testing $testname ... "