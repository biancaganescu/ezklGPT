#! /usr/bin/bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

source "$HOME/.cargo/env"

rustup override set nightly-2022-11-04