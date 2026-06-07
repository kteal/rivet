# rivet

[![CI](https://github.com/kteal/rivet/actions/workflows/ci.yml/badge.svg)](https://github.com/kteal/rivet/actions/workflows/ci.yml)
![Nix Flake](https://img.shields.io/badge/Nix-Flake-5277C3?logo=nixos)
![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)

`rivet` is a C compiler written in Rust that targets RV32IM assembly. It is working toward a C23 implementation and currently implements a small C23 subset.

It currently supports a small C subset with `int`, `char`, `unsigned int`, and basic pointer types, local variables, functions, calls, block scope, common control flow, assignment expressions, compound assignments, prefix/postfix increment and decrement, dereference expressions, and most integer operators with C-like precedence. It emits RV32IM assembly and reports lexer, parser, and semantic errors with source locations.

The current language subset supports programs shaped like:

```c
int triangular_until(int x, int stop) {
    int sum = 0;

    for (int i = x; i > 0; i--) {
        if (i == stop) {
            break;
        }

        if (i == 2) {
            continue;
        }

        sum = sum + i;
    }

    return sum;
}

int adjust(int value, int mask) {
    char newline = '\n';

    do {
        value++;
    } while (value < 6);

    if ((value & mask) == 6 && newline == 10) {
        return value;
    } else {
        return 0;
    }
}

int main() {
    int sum;
    sum = triangular_until(5, 1);
    adjust(sum, 7);
    return sum = adjust(sum, 7);
}
```

## Development Environment

### Nix (recommended)

The provided Nix development shell includes:

- Rust toolchain (`cargo`, `rustc`, `clippy`, `rustfmt`, `rust-analyzer`, `cargo-nextest`)
- RISC-V assembler and linker
- QEMU user emulation

Enter the shell with:

```bash
nix develop
```

### Without Nix

Install:

- Rust and Cargo
- `riscv64-linux-gnu-as`
- `riscv64-linux-gnu-ld`
- `qemu-riscv32`

On Ubuntu or Debian:

```bash
sudo apt install qemu-user binutils-riscv64-linux-gnu
```

All commands below assume either:

- you are inside the Nix development shell, or
- the required tools are installed manually.

## Build

```bash
cargo build
```

or build the Nix package, which also runs the package check phase:

```bash
nix build
```

## Formatting

```bash
cargo fmt
```

or with Nix

```bash
nix fmt
```

## Generate Assembly

`rivet` reads a `.c` file and prints RV32IM assembly to stdout.

```bash
cargo run -- path/to/program.c
```

Example:

```bash
cargo run -- tests/smoke.c
```

Lex, parse, and semantic errors are reported with file, line, and column information:

```text
path/to/program.c:2:12: error: undeclared local variable 'x'
```

## Run Tests

The repository includes Rust unit tests as well as end-to-end QEMU tests. These compile source programs, assemble and link the generated RV32 output, then run the result under `qemu-riscv32`.

```bash
cargo nextest run --locked --all-targets --all-features
```

CI runs tests inside the Nix development shell so the RISC-V binutils and QEMU tools are available:

```bash
nix develop --command cargo nextest run --locked --all-targets --all-features
```

To run the same Nix checks used by CI:

```bash
nix build
nix flake check
```

## Run Generated Programs Under QEMU

Use the helper script:

```bash
scripts/run-rv32.sh path/to/program.c
```

Or assert an expected exit code:

```bash
scripts/run-rv32.sh --expect 7 path/to/program.c
```

The script:

1. runs `rivet` to produce assembly
2. assembles and links an RV32 executable
3. runs it with `qemu-riscv32`
4. prints the program exit code

## Status

Lexing and preprocessing:

- [x] integer literals
- [x] character constants
- [x] comments
- [ ] string literals
- [ ] preprocessing tokens and macro expansion
- [ ] `#include`
- [ ] conditional compilation

Program structure and declarations:

- [x] local variable declarations
- [x] `return`
- [x] blocks
- [x] nested blocks and scope
- [x] function definitions beyond `main`
- [x] declarations without initializers
- [ ] declaration lists mixed with statements
- [ ] multiple translation-unit-level declarations
- [ ] globals
- [ ] typedef names
- [ ] storage classes: `extern`, `static`, `auto`, `register`, `thread_local`
- [ ] qualifiers: `const`, `volatile`, `restrict`, `_Atomic`
- [ ] full C declarator grammar

Expressions and operators:

- [x] arithmetic: `+ - * / %`
- [x] operator precedence and left associativity
- [x] parenthesized expressions
- [x] assignment expressions
- [x] unary operators: `- ! ~`
- [x] bitwise operators: `& | ^ << >>`
- [x] comparisons: `== != < <= > >=`
- [x] C-style left-associative chained comparisons
- [x] expression statements
- [x] empty statements
- [x] logical `&&` and `||` with short-circuiting
- [x] compound assignments: `+= -= *= /= %= &= |= ^= <<= >>=`
- [x] prefix and postfix `++` / `--`
- [x] pointer dereference as an rvalue: `*p`
- [ ] conditional operator `?:`
- [ ] comma operator
- [ ] casts
- [ ] `sizeof`
- [ ] `_Alignof` / `alignof`
- [ ] address-of: `&`
- [ ] dereference as an lvalue: `*p = value`
- [ ] array-to-pointer and function-to-pointer decay

Types and semantic analysis:

- [x] semantic errors for undeclared and duplicate locals
- [x] basic type checking and implicit conversions for `int`, `char`, and `unsigned int`
- [x] `char`
- [x] `unsigned int`
- [x] basic pointer types such as `char *` and `int *`
- [x] pointer dereference type checking
- [x] pointer arithmetic with integer offsets
- [ ] full integer conversion rules
- [ ] remaining signedness spelling and combinations: `signed`, bare `unsigned`, `unsigned char`, etc.
- [ ] other non-`int` scalar types: `short`, `long`
- [ ] fixed-width and standard integer typedef compatibility
- [ ] `bool`, `true`, `false`
- [ ] enum types and enumerators

Control flow:

- [x] `if` / `else`
- [x] `while`
- [x] `break` and `continue`
- [x] `for`
- [x] `do` / `while`
- [ ] `switch`, `case`, and `default`
- [ ] `goto` and labels

Functions:

- [x] function definitions beyond `main`
- [x] zero-argument function calls
- [x] function parameters
- [x] function calls with up to 8 register arguments
- [x] register argument passing with `a0`-`a7`
- [ ] stack-passed function arguments beyond 8
- [ ] full call ABI handling

Objects, aggregate types, and declarators:

- [x] pointer parameters and local declarations
- [x] pointer arithmetic scaled by pointee size
- [ ] arrays
- [ ] array indexing
- [ ] full pointer lvalue support
- [ ] full C declarator grammar
- [ ] structs and unions
- [ ] member access: `.` and `->`
- [ ] initializer lists
- [ ] compound literals

Toolchain and library compatibility:

- [x] diagnostics with source locations
- [ ] standard header strategy
- [ ] minimal hosted C runtime integration
- [ ] standard library calls through external symbols
- [ ] warnings vs errors
- [ ] separate compilation and object files
- [ ] linker/assembler integration beyond the current assembly output

Backend and portability:

- [ ] RV64 target support
- [ ] RISC-V ABI coverage for calls, returns, stack alignment, and callee-saved registers
- [ ] register allocation
- [ ] intermediate representation
- [ ] basic optimization passes
