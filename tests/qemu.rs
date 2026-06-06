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
fn qemu_logical_operator_programs_return_expected_values() {
    run_qemu_case(
        "logical-and-true",
        "int main() {\n    return 2 && 3;\n}\n",
        1,
    );
    run_qemu_case(
        "logical-and-false-left",
        "int main() {\n    return 0 && 3;\n}\n",
        0,
    );
    run_qemu_case(
        "logical-and-false-right",
        "int main() {\n    return 2 && 0;\n}\n",
        0,
    );
    run_qemu_case(
        "logical-or-true-left",
        "int main() {\n    return 2 || 0;\n}\n",
        1,
    );
    run_qemu_case(
        "logical-or-true-right",
        "int main() {\n    return 0 || 3;\n}\n",
        1,
    );
    run_qemu_case(
        "logical-or-false",
        "int main() {\n    return 0 || 0;\n}\n",
        0,
    );
    run_qemu_case(
        "logical-and-normalizes-right",
        "int main() {\n    return 1 && 42;\n}\n",
        1,
    );
    run_qemu_case(
        "logical-or-normalizes-right",
        "int main() {\n    return 0 || 42;\n}\n",
        1,
    );
    run_qemu_case(
        "logical-and-before-or",
        "int main() {\n    return 0 || 1 && 2;\n}\n",
        1,
    );
    run_qemu_case(
        "bitwise-or-before-logical-and",
        "int main() {\n    return 1 | 0 && 0;\n}\n",
        0,
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
        "declaration-without-initializer",
        "int main() {\n    int x;\n    x = 3;\n    return x;\n}\n",
        3,
    );
    run_qemu_case(
        "assignment-expression-result",
        "int main() {\n    int x;\n    return x = 3;\n}\n",
        3,
    );
    run_qemu_case(
        "chained-assignment-expression",
        "int main() {\n    int x;\n    int y;\n    x = y = 4;\n    return x + y;\n}\n",
        8,
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
    run_qemu_case(
        "break-from-while",
        "int main() {\n    int x = 0;\n    while (1) {\n        x = x + 1;\n        if (x == 3) break;\n    }\n    return x;\n}\n",
        3,
    );
    run_qemu_case(
        "continue-in-while",
        "int main() {\n    int x = 0;\n    int sum = 0;\n    while (x < 5) {\n        x = x + 1;\n        if (x == 3) continue;\n        sum = sum + x;\n    }\n    return sum;\n}\n",
        12,
    );
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_for_programs_return_expected_values() {
    run_qemu_case(
        "for-countdown",
        "int main() {\n    int i = 0;\n    for (i = 0; i < 3; i = i + 1) {\n    }\n    return i;\n}\n",
        3,
    );
    run_qemu_case(
        "for-sum-with-decl-init",
        "int main() {\n    int sum = 0;\n    for (int i = 1; i < 4; i = i + 1) {\n        sum = sum + i;\n    }\n    return sum;\n}\n",
        6,
    );
    run_qemu_case(
        "for-empty-condition-break",
        "int main() {\n    int i = 0;\n    for (;;) {\n        i = i + 1;\n        if (i == 4) break;\n    }\n    return i;\n}\n",
        4,
    );
    run_qemu_case(
        "continue-in-for-runs-post",
        "int main() {\n    int sum = 0;\n    for (int i = 0; i < 5; i = i + 1) {\n        if (i == 3) continue;\n        sum = sum + i;\n    }\n    return sum;\n}\n",
        7,
    );
    run_qemu_case(
        "for-init-shadows-outer-local",
        "int main() {\n    int i = 5;\n    for (int i = 0; i < 1; i = i + 1) {\n    }\n    return i;\n}\n",
        5,
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
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_expression_and_empty_statements_return_expected_values() {
    run_qemu_case(
        "empty-statement",
        "int main() {\n    ;\n    return 7;\n}\n",
        7,
    );
    run_qemu_case(
        "call-expression-statement",
        "int helper() {\n    return 3;\n}\n\nint main() {\n    helper();\n    return 7;\n}\n",
        7,
    );
    run_qemu_case(
        "literal-expression-statement",
        "int main() {\n    1 + 2;\n    return 5;\n}\n",
        5,
    );
}
