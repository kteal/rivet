# rivet

[![CI](https://github.com/kteal/rivet/actions/workflows/ci.yml/badge.svg)](https://github.com/kteal/rivet/actions/workflows/ci.yml)
![Nix Flake](https://img.shields.io/badge/Nix-Flake-5277C3?logo=nixos)
![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)

`rivet` is a C compiler written in Rust that targets RV32IM assembly. It is working toward C23 by growing a small, tested C subset.

It currently handles common integer and character types, basic pointers, function pointers, string literal expressions, address-of and dereference, fixed-size local and file-scope arrays, file-scope globals, array indexing, `sizeof`, functions, simple function prototypes, block scope, common control flow, and C-like expression precedence. It emits RV32IM assembly and reports lexer, parser, and semantic errors with source-map-backed file, line, and column locations.

The current language subset supports programs shaped like:

```c
int sum3(char *p) {
    int sum = 0;

    for (int i = 0; i < 3; i++) {
        sum += *p;
        p++;
    }

    return sum;
}

int main() {
    char buf[3] = {'a', 'b', 'c'};
    return sum3(buf);
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

## Linting

```bash
cargo clippy --lib -p rivet --tests -- -W clippy::all -W clippy::pedantic -W clippy::nursery
```

## Generate Assembly

`rivet` reads a `.c` file and prints RV32IM assembly to stdout.

```bash
cargo run -- path/to/program.c
```

Example:

```bash
cargo run -- tests/programs/adler/full_harness.c
```

Lex, parse, and semantic errors are reported with file, line, and column information:

```text
path/to/program.c:2:12: error: undeclared variable 'x'
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
- [x] integer literal suffixes: `U`, `L`, `UL`
- [x] hexadecimal integer literals
- [x] character constants
- [x] comments
- [x] escaped-newline splicing for continued preprocessing directives
- [x] preprocessing tokens needed for object-like macros
- [x] object-like `#define` macros, including empty replacements
- [x] simple function-like `#define` macros with argument substitution and nested expansion
- [x] conditional compilation with `#ifdef`, `#ifndef`, `#else`, and `#endif`
- [x] local quoted `#include "file.h"` handling
- [x] string literal preprocessing tokens for quoted include paths
- [x] file-aware token spans with `SourceMap` / `FileId`
- [x] byte-backed string literal tokens with basic escape decoding
- [ ] full macro expansion semantics: hide sets, stringification, token pasting, variadics, and exact whitespace-sensitive function-like macro definition rules
- [ ] full `#include` behavior: system includes and include search paths

Program structure and declarations:

- [x] local variable declarations
- [x] `return`
- [x] blocks
- [x] nested blocks and scope
- [x] declarations without initializers
- [x] multiple local declarators in one declaration
- [x] declaration lists mixed with statements
- [x] multiple translation-unit-level declarations
- [x] file-scope globals
- [x] scalar and fixed-size array global initializers with zero-fill
- [x] top-level typedef aliases with comma-separated declarators
- [x] scoped typedef names with block, parameter, and `for`-scope object-name shadowing
- [ ] full typedef behavior: alias-preserving diagnostics and complete C compatibility
- [ ] storage classes: `extern`, `static`, `auto`, `register`, `thread_local`
- [x] ignored `const` qualifier parsing
- [ ] remaining qualifiers: `volatile`, `restrict`, `_Atomic`
- [x] parenthesized pointer-to-array declarators: `int (*p)[3]`
- [x] parenthesized function pointer declarators: `int (*fp)(int)`
- [x] pointer abstract type-names in casts and `sizeof`: `char *`, `int **`
- [ ] full C declarator grammar

Expressions and operators:

- [x] arithmetic: `+ - * / %`
- [x] operator precedence and left associativity
- [x] parenthesized expressions
- [x] assignment expressions
- [x] unary operators: `- ! ~ & *`
- [x] bitwise operators: `& | ^ << >>`
- [x] comparisons: `== != < <= > >=`
- [x] C-style left-associative chained comparisons
- [x] expression statements
- [x] empty statements
- [x] logical `&&` and `||` with short-circuiting
- [x] compound assignments: `+= -= *= /= %= &= |= ^= <<= >>=`
- [x] prefix and postfix `++` / `--`
- [x] pointer dereference as an rvalue: `*p`
- [x] pointer dereference as an lvalue: `*p = value`
- [x] array indexing as rvalue and lvalue: `a[i]`
- [x] null pointer constants in pointer assignment, calls, returns, and comparisons
- [x] compatible pointer equality and inequality
- [x] string literal expressions with array-to-pointer decay
- [ ] conditional operator `?:`
- [ ] comma operator
- [x] scalar casts
- [x] `sizeof` for supported scalar, pointer, array, and function-designator expression types
- [ ] `_Alignof` / `alignof`
- [x] address-of: `&`
- [x] array-to-pointer decay for local and global array expressions
- [x] function-to-pointer decay for function designators
- [x] explicit typed AST conversion nodes for lvalue-to-rvalue, array-to-pointer, and function-to-pointer conversions

Types and semantic analysis:

- [x] semantic errors for undeclared and duplicate locals
- [x] basic type checking and implicit conversions for `int`, `char`, and `unsigned int`
- [x] `char`
- [x] `signed char`
- [x] `unsigned char`
- [x] `unsigned int`
- [x] bare `unsigned` as `unsigned int`
- [x] `long`
- [x] `unsigned long`
- [x] `signed`, `signed int`, `signed long`, and `signed long int`
- [x] basic pointer types such as `char *` and `int *`
- [x] pointer dereference type checking
- [x] pointer arithmetic with integer offsets
- [ ] full integer conversion rules
- [ ] remaining signedness spelling and combinations
- [ ] other non-`int` scalar types: `short`, `unsigned short`
- [x] project-local typedef compatibility for integer aliases
- [ ] standard-library and fixed-width typedef compatibility
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
- [x] simple function declarations/prototypes
- [x] unnamed scalar and pointer parameters in function prototypes
- [x] zero-argument function calls
- [x] function parameters
- [x] function calls with up to 8 register arguments
- [x] register argument passing with `a0`-`a7`
- [x] function pointer calls, including `fp(args)` and `(*fp)(args)`
- [ ] stack-passed function arguments beyond 8
- [ ] full call ABI handling

Objects, aggregate types, and declarators:

- [x] pointer parameters and local declarations
- [x] function declarators for top-level functions
- [x] pointer arithmetic scaled by pointee size
- [x] fixed-size local array declarations and stack allocation
- [x] scalar initializer lists with zero-fill for local arrays
- [x] fixed-size global array declarations and data emission
- [x] scalar initializer lists with zero-fill for global arrays
- [x] trailing commas in initializer lists
- [x] array indexing
- [x] address-of arrays in semantic analysis: `&arr` has pointer-to-array type
- [x] parenthesized pointer-to-array declarators and indexing through them: `(*p)[i]`
- [x] function pointer declarators, typedefs, initialization from function designators, and indirect calls
- [x] pointer abstract type-names for `sizeof(type-name)` and casts
- [x] static `.rodata` storage for string literal expressions
- [x] string literal initialization for explicit-size character arrays: `char buf[4] = "abc"`
- [x] inferred-size character arrays from string literals: `char buf[] = "abc"`
- [ ] adjacent string literal concatenation: `"foo" "bar"`
- [ ] full C declarator grammar
- [ ] structs and unions
- [ ] member access: `.` and `->`
- [ ] compound literals

Toolchain and library compatibility:

- [x] diagnostics with source-map-backed file, line, and column locations
- [x] full Adler-32 compatibility fixture with reduced local `zutil.h`
- [ ] macro expansion provenance in diagnostics
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
