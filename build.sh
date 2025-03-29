#!/usr/bin/env bash
cargo run -- -dump-ir2 examples/arrays.cpp &>/dev/null | tee -a ir2.asm
