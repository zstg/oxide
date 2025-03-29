#!/usr/bin/env bash
cargo run -- -dump-ir2 examples/fib.c &>/dev/null | tee -a out.asm
