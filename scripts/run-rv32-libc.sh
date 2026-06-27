#!/usr/bin/env bash
set -euo pipefail

expected_status=""

if [ "$#" -eq 3 ] && [ "$1" = "--expect" ]; then
    expected_status="$2"
    shift 2
fi

if [ "$#" -ne 1 ]; then
    echo "usage: $0 [--expect <exit-code>] <source.c>" >&2
    exit 2
fi

source_file="$1"

if [ ! -f "$source_file" ]; then
    echo "error: source file not found: $source_file" >&2
    exit 1
fi

for tool in cargo riscv32-unknown-linux-gnu-gcc qemu-riscv32; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: missing required tool: $tool" >&2
        echo "run this script inside: nix develop" >&2
        exit 1
    fi
done

libc_file="$(riscv32-unknown-linux-gnu-gcc -print-file-name=libc.so.6)"
if [ "$libc_file" = "libc.so.6" ] || [ ! -f "$libc_file" ]; then
    echo "error: failed to locate riscv32 glibc through riscv32-unknown-linux-gnu-gcc" >&2
    exit 1
fi

glibc_root="$(dirname "$(dirname "$libc_file")")"

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

asm_file="$workdir/program.s"
exe_file="$workdir/program"

cargo run --quiet -- "$source_file" > "$asm_file"

riscv32-unknown-linux-gnu-gcc \
    -march=rv32imafd \
    -mabi=ilp32d \
    -no-pie \
    -o "$exe_file" \
    "$asm_file"

set +e
qemu-riscv32 -L "$glibc_root" "$exe_file"
status="$?"
set -e

echo "$status"

if [ -n "$expected_status" ]; then
    if [ "$status" -eq "$expected_status" ]; then
        exit 0
    fi

    echo "error: expected exit code $expected_status, got $status" >&2
    exit 1
fi

exit "$status"
