#!/bin/bash

cargo build

export GIT="git"
export CGIT="./target/debug/chibigit"
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