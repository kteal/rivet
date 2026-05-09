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

for tool in cargo riscv64-linux-gnu-as riscv64-linux-gnu-ld qemu-riscv32; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: missing required tool: $tool" >&2
        echo "on Ubuntu/Debian, install: sudo apt install qemu-user binutils-riscv64-linux-gnu" >&2
        exit 1
    fi
done

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

asm_file="$workdir/program.s"
start_file="$workdir/start.s"
program_obj="$workdir/program.o"
start_obj="$workdir/start.o"
exe_file="$workdir/program"

cargo run --quiet -- "$source_file" > "$asm_file"

cat > "$start_file" <<'EOF'
.globl _start
_start:
    call main
    li a7, 93
    ecall
EOF

riscv64-linux-gnu-as -march=rv32im -mabi=ilp32 -o "$program_obj" "$asm_file"
riscv64-linux-gnu-as -march=rv32im -mabi=ilp32 -o "$start_obj" "$start_file"
riscv64-linux-gnu-ld -m elf32lriscv -o "$exe_file" "$start_obj" "$program_obj"

set +e
qemu-riscv32 "$exe_file"
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
