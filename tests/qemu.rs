use std::fs;
use std::process::Command;

fn run_qemu_case(name: &str, source: &str, expected: i32) {
    let tempdir = tempfile::tempdir().expect("failed to create temporary directory");
    let path = tempdir.path().join(format!("{name}.c"));

    fs::write(&path, source).expect("failed to write temporary source file");

    let status = Command::new("scripts/run-rv32.sh")
        .arg("--expect")
        .arg(expected.to_string())
        .arg(&path)
        .status()
        .expect("failed to run scripts/run-rv32.sh");

    assert!(status.success(), "qemu runner failed for {name}");
}

fn run_compile_error_case(name: &str, source: &str, expected_stderr: &str) {
    let tempdir = tempfile::tempdir().expect("failed to create temporary directory");
    let path = tempdir.path().join(format!("{name}.c"));

    fs::write(&path, source).expect("failed to write temporary source file");

    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--")
        .arg(&path)
        .output()
        .expect("failed to run compiler");

    assert!(
        !output.status.success(),
        "compiler unexpectedly succeeded for {name}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_stderr),
        "expected stderr for {name} to contain {expected_stderr:?}, got {stderr:?}"
    );
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_return_and_arithmetic_programs_return_expected_values() {
    run_qemu_case("return-42", "int main() {\n    return 42;\n}\n", 42);
    run_qemu_case("precedence", "int main() {\n    return 1 + 2 * 3;\n}\n", 7);
    run_qemu_case(
        "parentheses",
        "int main() {\n    return (1 + 2) * 3;\n}\n",
        9,
    );
    run_qemu_case("div-rem", "int main() {\n    return 8 / 2 + 8 % 3;\n}\n", 6);
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_comparison_programs_return_expected_values() {
    run_qemu_case("equal-true", "int main() {\n    return 5 == 5;\n}\n", 1);
    run_qemu_case("equal-false", "int main() {\n    return 5 == 3;\n}\n", 0);
    run_qemu_case("not-equal-true", "int main() {\n    return 5 != 3;\n}\n", 1);
    run_qemu_case(
        "not-equal-false",
        "int main() {\n    return 5 != 5;\n}\n",
        0,
    );
    run_qemu_case("less-true", "int main() {\n    return 2 < 5;\n}\n", 1);
    run_qemu_case("less-false", "int main() {\n    return 5 < 2;\n}\n", 0);
    run_qemu_case(
        "less-equal-true",
        "int main() {\n    return 5 <= 5;\n}\n",
        1,
    );
    run_qemu_case(
        "less-equal-false",
        "int main() {\n    return 6 <= 5;\n}\n",
        0,
    );
    run_qemu_case("greater-true", "int main() {\n    return 5 > 2;\n}\n", 1);
    run_qemu_case("greater-false", "int main() {\n    return 2 > 5;\n}\n", 0);
    run_qemu_case(
        "greater-equal-true",
        "int main() {\n    return 5 >= 5;\n}\n",
        1,
    );
    run_qemu_case(
        "greater-equal-false",
        "int main() {\n    return 2 >= 5;\n}\n",
        0,
    );
    run_qemu_case(
        "comparison-precedence",
        "int main() {\n    return 1 + 2 < 4;\n}\n",
        1,
    );
    run_qemu_case(
        "chained-comparison-true",
        "int main() {\n    return 1 < 2 < 3;\n}\n",
        1,
    );
    run_qemu_case(
        "chained-comparison-c-left-assoc",
        "int main() {\n    return 3 < 2 < 1;\n}\n",
        1,
    );
    run_qemu_case(
        "chained-comparison-false",
        "int main() {\n    return 3 < 2 < 0;\n}\n",
        0,
    );
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_unary_programs_return_expected_values() {
    run_qemu_case("unary-negation", "int main() {\n    return -5;\n}\n", 251);
    run_qemu_case("logical-not-zero", "int main() {\n    return !0;\n}\n", 1);
    run_qemu_case(
        "logical-not-nonzero",
        "int main() {\n    return !5;\n}\n",
        0,
    );
    run_qemu_case("bitwise-not", "int main() {\n    return ~0;\n}\n", 255);
    run_qemu_case(
        "unary-combined",
        "int main() {\n    return !0 + !!5 + ~0 + -3;\n}\n",
        254,
    );
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_bitwise_and_shift_programs_return_expected_values() {
    run_qemu_case("bitwise-and", "int main() {\n    return 6 & 3;\n}\n", 2);
    run_qemu_case("bitwise-xor", "int main() {\n    return 6 ^ 3;\n}\n", 5);
    run_qemu_case("bitwise-or", "int main() {\n    return 4 | 1;\n}\n", 5);
    run_qemu_case("shift-left", "int main() {\n    return 3 << 2;\n}\n", 12);
    run_qemu_case("shift-right", "int main() {\n    return 16 >> 2;\n}\n", 4);
    run_qemu_case(
        "signed-shift-right",
        "int main() {\n    return -8 >> 1;\n}\n",
        252,
    );
    run_qemu_case(
        "additive-before-shift",
        "int main() {\n    return 1 + 2 << 3;\n}\n",
        24,
    );
    run_qemu_case(
        "additive-before-shift-right",
        "int main() {\n    return 16 >> 2 + 1;\n}\n",
        2,
    );
    run_qemu_case(
        "shift-before-relational",
        "int main() {\n    return 1 << 2 < 8;\n}\n",
        1,
    );
    run_qemu_case(
        "equality-before-bitwise-and",
        "int main() {\n    return 5 & 3 == 1;\n}\n",
        0,
    );
    run_qemu_case(
        "parenthesized-bitwise-before-equality",
        "int main() {\n    return (5 & 3) == 1;\n}\n",
        1,
    );
    run_qemu_case(
        "bitwise-precedence-chain",
        "int main() {\n    return 1 | 2 ^ 3 & 1;\n}\n",
        3,
    );
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_local_variable_programs_return_expected_values() {
    run_qemu_case(
        "local-return",
        "int main() {\n    int x = 5;\n    return x;\n}\n",
        5,
    );
    run_qemu_case(
        "two-locals",
        "int main() {\n    int x = 5;\n    int y = x + 3;\n    return y;\n}\n",
        8,
    );
    run_qemu_case(
        "three-locals",
        "int main() {\n    int x = 2;\n    int y = x + 3;\n    int z = y * 4;\n    return z;\n}\n",
        20,
    );
    run_qemu_case(
        "four-locals",
        "int main() {\n    int a = 1;\n    int b = a + 2;\n    int c = b + 3;\n    int d = c + 4;\n    return d;\n}\n",
        10,
    );
    run_qemu_case(
        "assignment",
        "int main() {\n    int x = 1;\n    x = x + 2;\n    return x;\n}\n",
        3,
    );
    run_qemu_case(
        "multi-var-assignments",
        "int main() {\n    int x = 2;\n    int y = 3;\n    int z = x + y;\n    x = z * 2;\n    y = x - 1;\n    z = y % 4;\n    return z;\n}\n",
        1,
    );
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_block_scope_programs_return_expected_values() {
    run_qemu_case(
        "block-uses-outer-local",
        "int main() {\n    int x = 5;\n    {\n        return x;\n    }\n}\n",
        5,
    );
    run_qemu_case(
        "inner-shadowing",
        "int main() {\n    int x = 1;\n    {\n        int x = 2;\n        return x;\n    }\n}\n",
        2,
    );
    run_qemu_case(
        "outer-local-after-block",
        "int main() {\n    int x = 1;\n    {\n        int y = 2;\n    }\n    return x;\n}\n",
        1,
    );
    run_qemu_case(
        "nested-blocks",
        "int main() {\n    int x = 1;\n    {\n        int y = 2;\n        {\n            int z = 3;\n            return x + y + z;\n        }\n    }\n}\n",
        6,
    );
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_if_else_programs_return_expected_values() {
    run_qemu_case(
        "if-true",
        "int main() {\n    if (1) return 2;\n    return 3;\n}\n",
        2,
    );
    run_qemu_case(
        "if-false",
        "int main() {\n    if (0) return 2;\n    return 3;\n}\n",
        3,
    );
    run_qemu_case(
        "if-else-then",
        "int main() {\n    if (1) return 2; else return 3;\n}\n",
        2,
    );
    run_qemu_case(
        "if-else-else",
        "int main() {\n    if (0) return 2; else return 3;\n}\n",
        3,
    );
    run_qemu_case(
        "if-else-with-blocks",
        "int main() {\n    int x = 1;\n    if (x < 2) {\n        return x + 1;\n    } else {\n        return x + 2;\n    }\n}\n",
        2,
    );
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_while_programs_return_expected_values() {
    run_qemu_case(
        "while-countdown",
        "int main() {\n    int x = 3;\n    while (x) {\n        x = x - 1;\n    }\n    return x;\n}\n",
        0,
    );
    run_qemu_case(
        "while-sum",
        "int main() {\n    int x = 3;\n    int sum = 0;\n    while (x) {\n        sum = sum + x;\n        x = x - 1;\n    }\n    return sum;\n}\n",
        6,
    );
    run_qemu_case(
        "while-body-locals-counted-in-frame",
        "int main() {\n    int x = 1;\n    while (x) {\n        int a = 1;\n        int b = 2;\n        int c = 3;\n        x = x - 1;\n        return a + b + c;\n    }\n    return 0;\n}\n",
        6,
    );
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_multiple_function_definitions_return_expected_values() {
    run_qemu_case(
        "unused-helper-before-main",
        "int helper() {\n    return 3;\n}\n\nint main() {\n    return 7;\n}\n",
        7,
    );
    run_qemu_case(
        "unused-helper-after-main",
        "int main() {\n    return 11;\n}\n\nint helper() {\n    return 4;\n}\n",
        11,
    );
    run_qemu_case(
        "independent-function-locals",
        "int first() {\n    int x = 1;\n    return x;\n}\n\nint main() {\n    int x = 9;\n    return x;\n}\n",
        9,
    );
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_function_calls_return_expected_values() {
    run_qemu_case(
        "zero-arg-call",
        "int helper() {\n    return 3;\n}\n\nint main() {\n    return helper();\n}\n",
        3,
    );
    run_qemu_case(
        "forward-call",
        "int main() {\n    return helper();\n}\n\nint helper() {\n    return 5;\n}\n",
        5,
    );
    run_qemu_case(
        "call-result-in-expression",
        "int helper() {\n    return 3;\n}\n\nint main() {\n    return helper() + 2;\n}\n",
        5,
    );
    run_qemu_case(
        "single-argument-call",
        "int id(int x) {\n    return x;\n}\n\nint main() {\n    return id(7);\n}\n",
        7,
    );
    run_qemu_case(
        "two-argument-call",
        "int add(int x, int y) {\n    return x + y;\n}\n\nint main() {\n    return add(2, 3);\n}\n",
        5,
    );
    run_qemu_case(
        "expression-argument-call",
        "int add(int x, int y) {\n    return x + y;\n}\n\nint main() {\n    return add(1 + 2, 3 + 4);\n}\n",
        10,
    );
}

#[test]
#[ignore = "requires cargo"]
fn qemu_semantic_errors_are_reported_before_codegen() {
    run_compile_error_case(
        "semantic-undeclared-return",
        "int main() {\n    return x;\n}\n",
        "semantic analysis error: undeclared local variable 'x'",
    );
    run_compile_error_case(
        "semantic-undeclared-assignment",
        "int main() {\n    x = 1;\n    return 0;\n}\n",
        "semantic analysis error: undeclared local variable 'x'",
    );
    run_compile_error_case(
        "semantic-duplicate-local",
        "int main() {\n    int x = 1;\n    int x = 2;\n    return x;\n}\n",
        "semantic analysis error: duplicate local variable 'x'",
    );
    run_compile_error_case(
        "semantic-later-local",
        "int main() {\n    int y = x;\n    int x = 1;\n    return y;\n}\n",
        "semantic analysis error: undeclared local variable 'x'",
    );
    run_compile_error_case(
        "semantic-block-local-after-scope",
        "int main() {\n    {\n        int x = 1;\n    }\n    return x;\n}\n",
        "semantic analysis error: undeclared local variable 'x'",
    );
    run_compile_error_case(
        "semantic-undeclared-function-call",
        "int main() {\n    return helper();\n}\n",
        "semantic analysis error: undeclared function 'helper'",
    );
}
