# rivet

`rivet` is a C compiler written in Rust that targets RV32IM assembly. It is working toward a C23 implementation and currently implements a small C23 subset.

It currently supports integer literals, local variables, assignments, blocks, expression statements, empty statements, `if` / `else`, `while`, `for`, `break`, `continue`, return statements, comments, arithmetic, unary, comparison, and bitwise operators with C-like precedence. It also supports multiple `int` functions, parameters, and calls with up to 8 `int` arguments passed in RISC-V argument registers.

The current language subset supports programs shaped like:

```c
int triangular_until(int x, int stop) {
    int sum = 0;

    for (int i = x; i > 0; i = i - 1) {
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
    if ((value & mask) == 6) {
        return value;
    } else {
        return 0;
    }
}

int main() {
    int sum = triangular_until(5, 1);
    adjust(sum, 7);
    return adjust(sum, 7);
}
```

## Build

```bash
cargo build
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

## Run Tests

Normal Rust tests:

```bash
cargo test
```

The repository also includes ignored end-to-end QEMU tests. These compile source programs, assemble and link the generated RV32 output, then run the result under `qemu-riscv32`.

```bash
cargo test --test qemu -- --ignored
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

## Tooling Requirements

For QEMU-backed runs and tests, the following tools must be installed:

- `cargo`
- `riscv64-linux-gnu-as`
- `riscv64-linux-gnu-ld`
- `qemu-riscv32`

On Ubuntu or Debian:

```bash
sudo apt install qemu-user binutils-riscv64-linux-gnu
```

## Status

Lexing and preprocessing:

- [x] integer literals
- [x] comments
- [ ] character constants
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
- [x] assignments
- [x] unary operators: `- ! ~`
- [x] bitwise operators: `& | ^ << >>`
- [x] comparisons: `== != < <= > >=`
- [x] C-style left-associative chained comparisons
- [x] expression statements
- [x] empty statements
- [ ] logical `&&` and `||` with short-circuiting
- [ ] conditional operator `?:`
- [ ] comma operator
- [ ] prefix and postfix `++` / `--`
- [ ] compound assignments: `+= -= *= /= %= &= |= ^= <<= >>=`
- [ ] casts
- [ ] `sizeof`
- [ ] `_Alignof` / `alignof`
- [ ] address-of and dereference: `&` and `*`
- [ ] array-to-pointer and function-to-pointer decay

Types and semantic analysis:

- [x] semantic errors for undeclared and duplicate locals
- [ ] type checking and implicit conversions
- [ ] full integer conversion rules
- [ ] signedness: `signed`, `unsigned`
- [ ] non-`int` scalar types: `char`, `short`, `long`
- [ ] fixed-width and standard integer typedef compatibility
- [ ] `bool`, `true`, `false`
- [ ] enum types and enumerators

Control flow:

- [x] `if` / `else`
- [x] `while`
- [x] `break` and `continue`
- [x] `for`
- [ ] `do` / `while`
- [ ] `switch`, `case`, and `default`
- [ ] `goto` and labels

Functions:

- [x] function definitions beyond `main`
- [x] zero-argument function calls
- [x] function parameters
- [x] function calls with up to 8 `int` arguments
- [x] register argument passing with `a0`-`a7`
- [ ] stack-passed function arguments beyond 8
- [ ] full call ABI handling

Objects, aggregate types, and declarators:

- [ ] pointers
- [ ] pointer arithmetic
- [ ] arrays
- [ ] array indexing
- [ ] structs and unions
- [ ] member access: `.` and `->`
- [ ] initializer lists
- [ ] compound literals

Toolchain and library compatibility:

- [ ] standard header strategy
- [ ] minimal hosted C runtime integration
- [ ] standard library calls through external symbols
- [ ] diagnostics with source locations
- [ ] warnings vs errors
- [ ] separate compilation and object files
- [ ] linker/assembler integration beyond the current assembly output

Backend and portability:

- [ ] RV64 target support
- [ ] RISC-V ABI coverage for calls, returns, stack alignment, and callee-saved registers
- [ ] register allocation
- [ ] intermediate representation
- [ ] basic optimization passes
