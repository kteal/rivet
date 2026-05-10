#!/usr/bin/env bash
set -euo pipefail

keep_workdir=0
gcc_tool="${RISCV_GCC:-riscv64-linux-gnu-gcc}"

while [ "$#" -gt 0 ]; do
    case "$1" in
        --keep)
            keep_workdir=1
            shift
            ;;
        --gcc)
            if [ "$#" -lt 2 ]; then
                echo "error: --gcc requires a tool name" >&2
                exit 2
            fi
            gcc_tool="$2"
            shift 2
            ;;
        --help|-h)
            echo "usage: $0 [--keep] [--gcc <riscv-gcc>] <source.c>" >&2
            exit 0
            ;;
        --)
            shift
            break
            ;;
        -*)
            echo "error: unknown option: $1" >&2
            echo "usage: $0 [--keep] [--gcc <riscv-gcc>] <source.c>" >&2
            exit 2
            ;;
        *)
            break
            ;;
    esac
done

if [ "$#" -ne 1 ]; then
    echo "usage: $0 [--keep] [--gcc <riscv-gcc>] <source.c>" >&2
    exit 2
fi

source_file="$1"

if [ ! -f "$source_file" ]; then
    echo "error: source file not found: $source_file" >&2
    exit 1
fi

for tool in cargo riscv64-linux-gnu-as riscv64-linux-gnu-ld qemu-riscv32 "$gcc_tool"; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "error: missing required tool: $tool" >&2
        echo "on Ubuntu/Debian, install: sudo apt install qemu-user binutils-riscv64-linux-gnu gcc-riscv64-linux-gnu" >&2
        exit 1
    fi
done

workdir="$(mktemp -d)"
if [ "$keep_workdir" -eq 1 ]; then
    echo "workdir: $workdir" >&2
else
    trap 'rm -rf "$workdir"' EXIT
fi

start_file="$workdir/start.s"

rivet_asm="$workdir/rivet.s"
rivet_obj="$workdir/rivet.o"
rivet_start_obj="$workdir/rivet-start.o"
rivet_exe="$workdir/rivet"

gcc_exe="$workdir/gcc"

cat > "$start_file" <<'EOF'
.globl _start
_start:
    call main
    li a7, 93
    ecall
EOF

cargo run --quiet -- "$source_file" > "$rivet_asm"
riscv64-linux-gnu-as -march=rv32im -mabi=ilp32 -o "$rivet_obj" "$rivet_asm"
riscv64-linux-gnu-as -march=rv32im -mabi=ilp32 -o "$rivet_start_obj" "$start_file"
riscv64-linux-gnu-ld -m elf32lriscv -o "$rivet_exe" "$rivet_start_obj" "$rivet_obj"

"$gcc_tool" \
    -std=c2x \
    -march=rv32im \
    -mabi=ilp32 \
    -ffreestanding \
    -nostdlib \
    -nostartfiles \
    -static \
    -fno-pic \
    -fno-pie \
    -Wl,-m,elf32lriscv \
    "$start_file" \
    "$source_file" \
    -o "$gcc_exe"

set +e
qemu-riscv32 "$rivet_exe"
rivet_status="$?"
qemu-riscv32 "$gcc_exe"
gcc_status="$?"
set -e

echo "rivet: $rivet_status"
echo "gcc:   $gcc_status"

if [ "$rivet_status" -eq "$gcc_status" ]; then
    exit 0
fi

echo "error: exit codes differ" >&2
exit 1
