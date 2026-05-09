# rivet

`rivet` is a C compiler written in Rust that targets RV32IM assembly. It implements a small subset of the C23 standard.

It is currently a small expression-and-local-variable subset: integer literals, local variables, assignments, blocks, return statements, comments, arithmetic, unary, comparison, and bitwise operators with C-like precedence.

The current language subset supports programs shaped like:

```c
int main() {
    int x = 2;
    int y = 3;
    int z = x + y;
    x = z * 2;
    y = x - 1;
    z = (y & 7) << 1;
    z = z == 10;
    return z;
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

## Project Layout

- [src/lexer.rs](/home/kteal/sources/rivet/src/lexer.rs) tokenizes source text
- [src/parser.rs](/home/kteal/sources/rivet/src/parser.rs) builds the AST
- [src/ast.rs](/home/kteal/sources/rivet/src/ast.rs) defines the compiler IR so far
- [src/codegen.rs](/home/kteal/sources/rivet/src/codegen.rs) emits RV32 assembly
- [scripts/run-rv32.sh](/home/kteal/sources/rivet/scripts/run-rv32.sh) runs generated code under QEMU
- [tests/qemu.rs](/home/kteal/sources/rivet/tests/qemu.rs) contains ignored end-to-end behavior tests

## Status

- [x] integer literals
- [x] local variable declarations
- [x] assignments
- [x] `return`
- [x] arithmetic: `+ - * / %`
- [x] operator precedence and left associativity
- [x] parenthesized expressions
- [x] unary operators: `- ! ~`
- [x] bitwise operators: `& | ^ << >>`
- [x] comparisons: `== != < <= > >=`
- [x] C-style left-associative chained comparisons
- [x] comments
- [x] semantic errors for undeclared and duplicate locals
- [x] blocks
- [x] nested blocks and scope
- [ ] `if` / `else`
- [ ] `while`
- [ ] function definitions beyond `main`
- [ ] function parameters
- [ ] function calls
- [ ] argument passing and call ABI handling
- [ ] type checking and implicit conversions
- [ ] signedness: `signed`, `unsigned`
- [ ] non-`int` scalar types: `char`, `short`, `long`
- [ ] pointers
- [ ] arrays
- [ ] globals
- [ ] string literals
- [ ] RV64 target support
