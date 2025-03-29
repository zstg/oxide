#!/usr/bin/env bash
cargo run -- -dump-ir2 examples/fib.c > out.asm
if nix run nixpkgs#nasm -- -f elf64 out.asm -o out.o; then
  # gcc -o out out.o
  echo "Assembly produced successfully"
else
  echo "Error produced assembly"
fi
