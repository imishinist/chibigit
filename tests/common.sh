#!/bin/bash

cargo build

export GIT="git"
export CGIT="./target/debug/chibigit"
export RUST_BACKTRACE=1