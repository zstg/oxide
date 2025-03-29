#!/usr/bin/env bash
cargo run -- -dump-ir2 examples/arrays.cpp | tee -a ir2.asm
