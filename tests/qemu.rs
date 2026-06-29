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

fn run_qemu_file(path: &str, expected: i32) {
    let status = Command::new("scripts/run-rv32.sh")
        .arg("--expect")
        .arg(expected.to_string())
        .arg(path)
        .status()
        .expect("failed to run scripts/run-rv32.sh");

    assert!(status.success(), "qemu runner failed for {path}");
}

fn run_qemu_libc_file(path: &str, expected: i32) {
    let status = Command::new("scripts/run-rv32-libc.sh")
        .arg("--expect")
        .arg(expected.to_string())
        .arg(path)
        .status()
        .expect("failed to run scripts/run-rv32-libc.sh");

    assert!(status.success(), "hosted qemu runner failed for {path}");
}

fn run_qemu_libc_file_with_args(path: &str, expected: i32, args: &[&str]) -> String {
    let output = Command::new("scripts/run-rv32-libc.sh")
        .arg("--expect")
        .arg(expected.to_string())
        .arg(path)
        .arg("--")
        .args(args)
        .output()
        .expect("failed to run scripts/run-rv32-libc.sh");

    assert!(
        output.status.success(),
        "hosted qemu runner failed for {path}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("hosted qemu output was not utf-8")
}

#[test]
fn qemu_adler32_reduced_harness_returns_success() {
    run_qemu_file("tests/programs/adler/harness.c", 0);
}

#[test]
fn qemu_adler32_zlib_compat_returns_success() {
    run_qemu_file("tests/programs/adler/zlib_compat.c", 0);
}

#[test]
fn qemu_adler32_do16_macro_ladder_returns_success() {
    run_qemu_file("tests/programs/adler/do16.c", 0);
}

#[test]
fn qemu_adler32_full_harness_returns_success() {
    run_qemu_file("tests/programs/adler/full_harness.c", 0);
}

#[test]
fn qemu_inih_harness_returns_success() {
    run_qemu_libc_file("tests/programs/inih/harness.c", 0);
}

#[test]
fn qemu_inih_file_harness_returns_success() {
    run_qemu_libc_file("tests/programs/inih/file_harness.c", 0);
}

#[test]
fn qemu_wc_default_counts_sample_file() {
    let output = run_qemu_libc_file_with_args("tests/programs/wc/rivet_wc.c", 0, &[]);

    assert!(output.contains("4 6 37 tests/programs/wc/sample.txt\n0\n"));
}

#[test]
fn qemu_wc_accepts_line_count_option() {
    let output = run_qemu_libc_file_with_args(
        "tests/programs/wc/rivet_wc.c",
        0,
        &["-l", "tests/programs/wc/sample.txt"],
    );

    assert!(output.contains("4 tests/programs/wc/sample.txt\n0\n"));
}

#[test]
fn qemu_wc_accepts_multiple_files_and_total() {
    let output = run_qemu_libc_file_with_args(
        "tests/programs/wc/rivet_wc.c",
        0,
        &[
            "-lc",
            "tests/programs/wc/sample.txt",
            "tests/programs/wc/second.txt",
        ],
    );

    assert!(output.contains("4 37 tests/programs/wc/sample.txt\n"));
    assert!(output.contains("2 14 tests/programs/wc/second.txt\n"));
    assert!(output.contains("6 51 total\n0\n"));
}

#[test]
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
fn qemu_pointer_comparison_programs_return_expected_values() {
    run_qemu_case(
        "pointer-greater-true",
        "int main() {\n    char buf[4];\n    char *start = &buf[0];\n    char *end = &buf[3];\n    return end > start;\n}\n",
        1,
    );
    run_qemu_case(
        "pointer-less-equal-same-address",
        "int main() {\n    char buf[4];\n    char *start = &buf[0];\n    return start <= start;\n}\n",
        1,
    );
    run_qemu_case(
        "pointer-less-false",
        "int main() {\n    char buf[4];\n    char *start = &buf[0];\n    char *end = &buf[3];\n    return end < start;\n}\n",
        0,
    );
    run_qemu_case(
        "void-pointer-relational-comparison",
        "int main() {\n    char buf[4];\n    char *start = &buf[0];\n    void *end = &buf[3];\n    return end >= start;\n}\n",
        1,
    );
}

#[test]
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
fn qemu_cast_programs_return_expected_values() {
    run_qemu_case(
        "cast-to-unsigned-char-narrows",
        "int main() {\n    return (unsigned char)300;\n}\n",
        44,
    );
    run_qemu_case(
        "cast-to-signed-char-sign-extends",
        "int main() {\n    return (signed char)255;\n}\n",
        255,
    );
    run_qemu_case(
        "cast-controls-comparison-type",
        "int main() {\n    return (unsigned)-1 > 1;\n}\n",
        1,
    );
    run_qemu_case(
        "adler-shaped-unsigned-long-cast",
        "int main() {\n    unsigned long x = 65521UL;\n    return ((unsigned long)x << 1) != 0;\n}\n",
        1,
    );
    run_qemu_case(
        "const-qualified-cast",
        "int main() {\n    return (const unsigned char)511;\n}\n",
        255,
    );
    run_qemu_case(
        "pointer-integer-pointer-cast-round-trip",
        "int main() {\n    char *p = \"abc\";\n    unsigned long x = (unsigned long)p;\n    char *q = (char *)x;\n    return q[1];\n}\n",
        98,
    );
    run_qemu_case(
        "zero-integer-cast-to-null-pointer",
        "int main() {\n    unsigned long x = 0;\n    char *p = (char *)x;\n    return p == 0;\n}\n",
        1,
    );
    run_qemu_case(
        "object-pointer-to-object-pointer-cast",
        "int main() {\n    int value = 0;\n    int *ip = &value;\n    char *cp = (char *)ip;\n    return cp != 0;\n}\n",
        1,
    );
}

#[test]
fn qemu_sizeof_programs_return_expected_values() {
    run_qemu_case(
        "sizeof-char",
        "int main() {\n    return sizeof(char);\n}\n",
        1,
    );
    run_qemu_case(
        "sizeof-int",
        "int main() {\n    return sizeof(int);\n}\n",
        4,
    );
    run_qemu_case(
        "sizeof-unsigned-long",
        "int main() {\n    return sizeof(unsigned long);\n}\n",
        4,
    );
    run_qemu_case(
        "sizeof-pointer-type",
        "int main() {\n    return sizeof(char *);\n}\n",
        4,
    );
    run_qemu_case(
        "sizeof-pointer-expression",
        "int main() {\n    char *p;\n    return sizeof(p);\n}\n",
        4,
    );
    run_qemu_case(
        "sizeof-local-variable-expression",
        "int main() {\n    int x;\n    return sizeof x;\n}\n",
        4,
    );
    run_qemu_case(
        "sizeof-parenthesized-variable-expression",
        "int main() {\n    int x;\n    return sizeof(x);\n}\n",
        4,
    );
    run_qemu_case(
        "sizeof-local-int-array-expression",
        "int main() {\n    int nums[3];\n    return sizeof(nums);\n}\n",
        12,
    );
    run_qemu_case(
        "sizeof-local-char-array-expression",
        "int main() {\n    char buf[3];\n    return sizeof(buf);\n}\n",
        3,
    );
    run_qemu_case(
        "sizeof-global-array-expression",
        "int nums[4];\n\nint main() {\n    return sizeof(nums);\n}\n",
        16,
    );
    run_qemu_case(
        "sizeof-pointer-dereference-expression",
        "int main() {\n    char *p;\n    return sizeof(*p);\n}\n",
        1,
    );
    run_qemu_case(
        "sizeof-typedef-name",
        "typedef unsigned char Byte;\n\nint main() {\n    return sizeof(Byte);\n}\n",
        1,
    );
    run_qemu_case(
        "sizeof-object-name-shadows-typedef-name",
        "typedef int T;\n\nint main() {\n    char T;\n    return sizeof(T);\n}\n",
        1,
    );
    run_qemu_case(
        "sizeof-has-unary-precedence",
        "int main() {\n    int x;\n    return sizeof x + 1;\n}\n",
        5,
    );
    run_qemu_case(
        "sizeof-local-anonymous-struct",
        "int main() {\n    struct { int x; char y; } value;\n    return sizeof(value);\n}\n",
        8,
    );
    run_qemu_case(
        "sizeof-pointer-to-struct-typedef",
        "typedef struct { char *ptr; unsigned long num_left; } Ctx;\n\nint main() {\n    Ctx *p;\n    return sizeof(p);\n}\n",
        4,
    );
}

#[test]
fn qemu_struct_member_programs_return_expected_values() {
    run_qemu_case(
        "direct-struct-int-field-read-write",
        "int main() {\n    struct { int x; char y; } item;\n    item.x = 37;\n    return item.x;\n}\n",
        37,
    );
    run_qemu_case(
        "direct-struct-padded-field-offset",
        "int main() {\n    struct { char tag; int value; } item;\n    item.tag = 5;\n    item.value = 37;\n    return item.tag + item.value;\n}\n",
        42,
    );
    run_qemu_case(
        "pointer-struct-member-read-write",
        "typedef struct { int x; char y; } Item;\n\nint main() {\n    Item item;\n    Item *p = &item;\n    p->x = 40;\n    p->y = 2;\n    return p->x + p->y;\n}\n",
        42,
    );
    run_qemu_case(
        "inih-shaped-struct-pointer-fields",
        "typedef struct { char *ptr; unsigned long num_left; } Ctx;\n\nint main() {\n    Ctx ctx;\n    Ctx *p = &ctx;\n    p->ptr = \"abc\";\n    p->num_left = 3;\n    return p->ptr[1] + p->num_left;\n}\n",
        101,
    );
    run_qemu_case(
        "tagged-struct-local-member-read-write",
        "int main() {\n    struct Point { int x; char y; } p;\n    p.x = 7;\n    p.y = 3;\n    return p.x + p.y;\n}\n",
        10,
    );
    run_qemu_case(
        "tagged-struct-reused-global-and-local",
        "struct Point { int x; char y; } global;\n\nint main() {\n    struct Point local;\n    local.x = 5;\n    local.y = 2;\n    global.x = local.x;\n    global.y = local.y;\n    return global.x + global.y;\n}\n",
        7,
    );
    run_qemu_case(
        "tagged-struct-pointer-member-read-write",
        "struct Item { int x; char y; };\n\nint main() {\n    struct Item item;\n    struct Item *p = &item;\n    p->x = 30;\n    p->y = 12;\n    return p->x + p->y;\n}\n",
        42,
    );
}

#[test]
fn qemu_string_literal_programs_return_expected_values() {
    run_qemu_case(
        "string-literal-pointer-index",
        "int main() {\n    char *s = \"abc\";\n    return s[1];\n}\n",
        98,
    );
    run_qemu_case(
        "string-literal-direct-index",
        "int main() {\n    return \"abc\"[2];\n}\n",
        99,
    );
    run_qemu_case(
        "string-literal-trailing-nul",
        "int main() {\n    return \"abc\"[3];\n}\n",
        0,
    );
    run_qemu_case(
        "string-literal-escape-byte",
        "int main() {\n    char *s = \"a\\n\";\n    return s[1];\n}\n",
        10,
    );
    run_qemu_case(
        "sizeof-string-literal",
        "int main() {\n    return sizeof(\"abc\");\n}\n",
        4,
    );
    run_qemu_case(
        "string-literal-char-array-initializer",
        "int main() {\n    char buf[4] = \"abc\";\n    return buf[2];\n}\n",
        99,
    );
    run_qemu_case(
        "empty-string-literal-char-array-initializer",
        "int main() {\n    char buf[4] = \"\";\n    return buf[0];\n}\n",
        0,
    );
    run_qemu_case(
        "string-literal-array-initializer-is-writable",
        "int main() {\n    char buf[4] = \"abc\";\n    buf[0] = 'z';\n    return buf[0];\n}\n",
        122,
    );
    run_qemu_case(
        "string-literal-infers-char-array-size",
        "int main() {\n    char buf[] = \"abc\";\n    return sizeof(buf);\n}\n",
        4,
    );
    run_qemu_case(
        "string-literal-inferred-array-index",
        "int main() {\n    char buf[] = \"abc\";\n    return buf[2];\n}\n",
        99,
    );
    run_qemu_case(
        "string-literal-infers-global-char-array-size",
        "char buf[] = \"abc\";\nint main() {\n    return sizeof(buf);\n}\n",
        4,
    );
    run_qemu_case(
        "string-literal-global-inferred-array-index",
        "char buf[] = \"abc\";\nint main() {\n    return buf[2];\n}\n",
        99,
    );
    run_qemu_case(
        "empty-string-literal-infers-one-byte-array",
        "int main() {\n    char buf[] = \"\";\n    return sizeof(buf);\n}\n",
        1,
    );
    run_qemu_case(
        "adjacent-string-literal-pointer-index",
        "int main() {\n    char *s = \"foo\" \"bar\";\n    return s[3];\n}\n",
        98,
    );
    run_qemu_case(
        "sizeof-adjacent-string-literals",
        "int main() {\n    return sizeof(\"foo\" \"bar\");\n}\n",
        7,
    );
    run_qemu_case(
        "adjacent-string-literals-infer-char-array-size",
        "int main() {\n    char buf[] = \"foo\" \"bar\";\n    return sizeof(buf);\n}\n",
        7,
    );
}

#[test]
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
        "logical-and-pointer-true",
        "int main() {\n    char *p = \"abc\";\n    return 1 && p;\n}\n",
        1,
    );
    run_qemu_case(
        "logical-or-null-pointer-false",
        "int main() {\n    char *p = 0;\n    return p || 0;\n}\n",
        0,
    );
    run_qemu_case(
        "logical-or-pointer-short-circuits",
        "int main() {\n    char *p = \"abc\";\n    return p || *p;\n}\n",
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
fn qemu_unsigned_int_programs_return_expected_values() {
    run_qemu_case(
        "bare-unsigned-local-initializer",
        "int main() {\n    unsigned x = 42;\n    return x;\n}\n",
        42,
    );
    run_qemu_case(
        "bare-unsigned-parameter-and-return",
        "unsigned id(unsigned x) {\n    return x;\n}\n\nint main() {\n    return id(7);\n}\n",
        7,
    );
    run_qemu_case(
        "bare-unsigned-comparison-uses-unsigned-operands",
        "int main() {\n    unsigned x = 0 - 1;\n    return x > 1;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-local-initializer",
        "int main() {\n    unsigned int x = 42;\n    return x;\n}\n",
        42,
    );
    run_qemu_case(
        "unsigned-parameter-and-return",
        "unsigned int id(unsigned int x) {\n    return x;\n}\n\nint main() {\n    return id(7);\n}\n",
        7,
    );
    run_qemu_case(
        "unsigned-comparison-uses-unsigned-operands",
        "int main() {\n    unsigned int x = 0 - 1;\n    return x > 1;\n}\n",
        1,
    );
    run_qemu_case(
        "mixed-unsigned-comparison-converts-int",
        "int main() {\n    unsigned int x = 0 - 1;\n    int y = 1;\n    return x > y;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-less-equal-uses-unsigned-operands",
        "int main() {\n    unsigned int x = 0 - 1;\n    return x <= 1;\n}\n",
        0,
    );
    run_qemu_case(
        "unsigned-greater-equal-uses-unsigned-operands",
        "int main() {\n    unsigned int x = 0 - 1;\n    return x >= 1;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-divide-uses-divu",
        "int main() {\n    unsigned int x = 0 - 2;\n    return x / 2 == 2147483647;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-remainder-uses-remu",
        "int main() {\n    unsigned int x = 0 - 1;\n    return x % 2 == 1;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-shift-right-uses-logical-shift",
        "int main() {\n    unsigned int x = 0 - 8;\n    return (x >> 1) < x;\n}\n",
        1,
    );
    run_qemu_case(
        "signed-left-shift-right-with-unsigned-count-uses-arithmetic-shift",
        "int main() {\n    int x = -8;\n    unsigned int shift = 1;\n    return (x >> shift) < 0;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-compound-divide-uses-divu",
        "int main() {\n    unsigned int x = 0 - 2;\n    x /= 2;\n    return x == 2147483647;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-compound-remainder-uses-remu",
        "int main() {\n    unsigned int x = 0 - 1;\n    x %= 2;\n    return x;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-compound-shift-right-uses-logical-shift",
        "int main() {\n    unsigned int x = 0 - 8;\n    unsigned int original = x;\n    x >>= 1;\n    return x < original;\n}\n",
        1,
    );
    run_qemu_case(
        "signed-left-compound-shift-right-with-unsigned-count-uses-arithmetic-shift",
        "int main() {\n    int x = -8;\n    unsigned int shift = 1;\n    x >>= shift;\n    return x < 0;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-unary-negation-preserves-unsigned-type",
        "int main() {\n    unsigned int x = 1;\n    return -x > 1;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-bitwise-not-preserves-unsigned-type",
        "int main() {\n    unsigned int x = 0;\n    return ~x > 1;\n}\n",
        1,
    );
}

#[test]
fn qemu_typedef_programs_return_expected_values() {
    run_qemu_case(
        "typedef-scalar-and-pointer-aliases",
        "typedef unsigned long uLong;\ntypedef unsigned char Bytef;\n\nuLong first(const Bytef *buf) {\n    return buf[0];\n}\n\nint main() {\n    Bytef buf[1] = {'a'};\n    return first(buf);\n}\n",
        97,
    );

    run_qemu_case(
        "typedef-local-declaration",
        "typedef unsigned int uInt;\n\nint main() {\n    uInt x = 42U;\n    return x;\n}\n",
        42,
    );

    run_qemu_case(
        "local-typedef-shadows-outer-typedef",
        "typedef int T;\n\nint main() {\n    typedef char T;\n    T x = 250;\n    return x;\n}\n",
        250,
    );

    run_qemu_case(
        "typedef-scope-restored-after-block",
        "typedef int T;\n\nint main() {\n    {\n        typedef char T;\n        T x = 250;\n    }\n    T y = 300;\n    return y;\n}\n",
        44,
    );

    run_qemu_case(
        "object-name-shadows-typedef-name",
        "typedef int T;\n\nint main() {\n    int T = 3;\n    return T;\n}\n",
        3,
    );

    run_qemu_case(
        "parameter-name-shadows-typedef-name",
        "typedef int T;\n\nint id(int T) {\n    return T;\n}\n\nint main() {\n    return id(7);\n}\n",
        7,
    );

    run_qemu_case(
        "for-init-object-name-scope-restores-typedef",
        "typedef int T;\n\nint main() {\n    for (int T = 0; T < 1; T = T + 1) {\n    }\n    T y = 9;\n    return y;\n}\n",
        9,
    );
}

#[test]
fn qemu_object_like_macro_programs_return_expected_values() {
    run_qemu_case(
        "object-like-macro-constant",
        "#define BASE 42\nint main() {\n    return BASE;\n}\n",
        42,
    );

    run_qemu_case(
        "empty-object-like-macro",
        "#define ZEXPORT\nint ZEXPORT main() {\n    return 7;\n}\n",
        7,
    );

    run_qemu_case(
        "object-like-macro-null-pointer",
        "#define Z_NULL 0\nint main() {\n    char *p = Z_NULL;\n    return p == Z_NULL;\n}\n",
        1,
    );

    run_qemu_case(
        "object-like-macro-with-typedefs",
        "#define BASE 65521U\ntypedef unsigned long uLong;\ntypedef unsigned char Bytef;\n\nuLong first(Bytef *buf) {\n    return buf[0] % BASE;\n}\n\nint main() {\n    Bytef buf[1] = {'a'};\n    return first(buf);\n}\n",
        97,
    );
}

#[test]
fn qemu_function_like_macro_programs_return_expected_values() {
    run_qemu_case(
        "function-like-macro-add",
        "#define ADD(x, y) x + y\nint main() {\n    return ADD(2, 3);\n}\n",
        5,
    );

    run_qemu_case(
        "zero-arg-function-like-macro",
        "#define VALUE() 7\nint main() {\n    return VALUE();\n}\n",
        7,
    );

    run_qemu_case(
        "nested-object-like-macro",
        "#define A B\n#define B 11\nint main() {\n    return A;\n}\n",
        11,
    );

    run_qemu_case(
        "nested-function-like-macro",
        "#define DOUBLE(x) x + x\n#define QUAD(x) DOUBLE(x) + DOUBLE(x)\nint main() {\n    return QUAD(3);\n}\n",
        12,
    );

    run_qemu_case(
        "adler-shaped-do-macros",
        "#define DO1(buf, i) sum += buf[i]\n#define DO2(buf, i) DO1(buf, i); DO1(buf, i + 1)\nint main() {\n    unsigned char buf[2] = {5, 7};\n    unsigned int sum = 0;\n    DO2(buf, 0);\n    return sum;\n}\n",
        12,
    );

    run_qemu_case(
        "continued-function-like-macro",
        "#define ADD_TO_SUM(buf, i) \\\n    sum += buf[i]\nint main() {\n    unsigned char buf[1] = {7};\n    unsigned int sum = 0;\n    ADD_TO_SUM(buf, 0);\n    return sum;\n}\n",
        7,
    );
}

#[test]
fn qemu_long_programs_return_expected_values() {
    run_qemu_case(
        "long-local-initializer",
        "int main() {\n    long x = 42;\n    return x;\n}\n",
        42,
    );
    run_qemu_case(
        "long-int-local-initializer",
        "int main() {\n    long int x = 42;\n    return x;\n}\n",
        42,
    );
    run_qemu_case(
        "long-parameter-and-return",
        "long id(long x) {\n    return x;\n}\n\nint main() {\n    return id(7);\n}\n",
        7,
    );
    run_qemu_case(
        "long-comparison-uses-signed-operands",
        "int main() {\n    long x = 0 - 1;\n    return x < 1;\n}\n",
        1,
    );
    run_qemu_case(
        "long-shift-right-uses-arithmetic-shift",
        "int main() {\n    long x = 0 - 8;\n    return (x >> 1) < 0;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-long-local-initializer",
        "int main() {\n    unsigned long x = 42;\n    return x;\n}\n",
        42,
    );
    run_qemu_case(
        "unsigned-long-int-local-initializer",
        "int main() {\n    unsigned long int x = 42;\n    return x;\n}\n",
        42,
    );
    run_qemu_case(
        "unsigned-long-parameter-and-return",
        "unsigned long id(unsigned long x) {\n    return x;\n}\n\nint main() {\n    return id(7);\n}\n",
        7,
    );
    run_qemu_case(
        "unsigned-long-comparison-uses-unsigned-operands",
        "int main() {\n    unsigned long x = 0 - 1;\n    return x > 1;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-long-shift-right-uses-logical-shift",
        "int main() {\n    unsigned long x = 0 - 8;\n    return (x >> 1) < x;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-int-plus-long-promotes-to-unsigned-long",
        "int main() {\n    unsigned int x = 0 - 1;\n    long y = 0;\n    return x + y > 1;\n}\n",
        1,
    );
}

#[test]
fn qemu_hex_integer_literal_programs_return_expected_values() {
    run_qemu_case(
        "hex-literal-equals-decimal-value",
        "int main() {\n    return 0xff == 255;\n}\n",
        1,
    );
    run_qemu_case(
        "uppercase-hex-literal-with-unsigned-suffix",
        "int main() {\n    return 0XFFU == 255U;\n}\n",
        1,
    );
    run_qemu_case(
        "hex-literal-with-unsigned-long-suffix",
        "int main() {\n    return 0xffffUL == 65535U;\n}\n",
        1,
    );
    run_qemu_case(
        "hex-literal-with-long-unsigned-suffix",
        "int main() {\n    return 0xffffLU == 65535U;\n}\n",
        1,
    );
    run_qemu_case(
        "large-unsuffixed-hex-literal-uses-unsigned-int",
        "int main() {\n    return 0xffffffff > 1;\n}\n",
        1,
    );
}

#[test]
fn qemu_signed_type_spelling_programs_return_expected_values() {
    run_qemu_case(
        "signed-is-int",
        "int main() {\n    signed x = 42;\n    return x;\n}\n",
        42,
    );
    run_qemu_case(
        "signed-int-is-int",
        "int main() {\n    signed int x = 42;\n    return x;\n}\n",
        42,
    );
    run_qemu_case(
        "signed-long-is-long",
        "int main() {\n    signed long x = 0 - 1;\n    return x < 0;\n}\n",
        1,
    );
    run_qemu_case(
        "signed-long-int-is-long",
        "int main() {\n    signed long int x = 0 - 1;\n    return x < 0;\n}\n",
        1,
    );
}

#[test]
fn qemu_char_family_programs_return_expected_values() {
    run_qemu_case(
        "unsigned-char-zero-extends-local-load",
        "int main() {\n    unsigned char c = 255;\n    return c == 255;\n}\n",
        1,
    );
    run_qemu_case(
        "signed-char-sign-extends-local-load",
        "int main() {\n    signed char c = 255;\n    return c < 0;\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-char-parameter-zero-extends",
        "int is_255(unsigned char c) {\n    return c == 255;\n}\n\nint main() {\n    return is_255(255);\n}\n",
        1,
    );
    run_qemu_case(
        "signed-char-parameter-sign-extends",
        "int is_negative(signed char c) {\n    return c < 0;\n}\n\nint main() {\n    return is_negative(255);\n}\n",
        1,
    );
    run_qemu_case(
        "unsigned-char-array-dereference-zero-extends",
        "int main() {\n    unsigned char bytes[1] = {255};\n    unsigned char *p = bytes;\n    return *p == 255;\n}\n",
        1,
    );
    run_qemu_case(
        "signed-char-array-dereference-sign-extends",
        "int main() {\n    signed char bytes[1] = {255};\n    signed char *p = bytes;\n    return *p < 0;\n}\n",
        1,
    );
    run_qemu_case(
        "char-family-arithmetic-promotes-to-int",
        "int main() {\n    unsigned char a = 255;\n    signed char b = 1;\n    return a + b == 256;\n}\n",
        1,
    );
    run_qemu_case(
        "signed-char-shift-promotes-to-int",
        "int main() {\n    signed char c = 255;\n    return (c >> 1) < 0;\n}\n",
        1,
    );
}

#[test]
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
        "multiple-local-declarators",
        "int main() {\n    int a = 2, b = a + 3, c = b + a;\n    return c;\n}\n",
        7,
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
        "lvalue-to-rvalue-local-loads",
        "int main() {\n    int x = 4;\n    int y = 5;\n    return x * 10 + y;\n}\n",
        45,
    );
    run_qemu_case(
        "lvalue-to-rvalue-pointer-dereference-load",
        "int main() {\n    int x = 6;\n    int *p = &x;\n    return *p + x;\n}\n",
        12,
    );
    run_qemu_case(
        "multi-var-assignments",
        "int main() {\n    int x = 2;\n    int y = 3;\n    int z = x + y;\n    x = z * 2;\n    y = x - 1;\n    z = y % 4;\n    return z;\n}\n",
        1,
    );
}

#[test]
fn qemu_local_array_programs_return_expected_values() {
    run_qemu_case(
        "char-array-local-reserves-stack-space",
        "int main() {\n    char buf[3];\n    int x = 7;\n    return x;\n}\n",
        7,
    );
    run_qemu_case(
        "int-array-local-reserves-stack-space",
        "int main() {\n    int nums[10];\n    int x = 9;\n    return x;\n}\n",
        9,
    );
    run_qemu_case(
        "char-array-decays-to-pointer-argument",
        "int write_first(char *p) {\n    *p = 42;\n    return *p;\n}\n\nint main() {\n    char buf[3];\n    return write_first(buf);\n}\n",
        42,
    );
    run_qemu_case(
        "int-array-decays-to-pointer-argument",
        "int write_first(int *p) {\n    return *p = 123;\n}\n\nint main() {\n    int nums[3];\n    return write_first(nums);\n}\n",
        123,
    );
    run_qemu_case(
        "char-array-decays-to-pointer-initializer",
        "int main() {\n    char buf[2] = {5, 0};\n    char *p = buf;\n    return *p;\n}\n",
        5,
    );
    run_qemu_case(
        "char-array-decays-to-pointer-comparison",
        "int main() {\n    char buf[2];\n    char *p = buf;\n    return p == buf;\n}\n",
        1,
    );
    run_qemu_case(
        "char-array-initializer-list",
        "int sum3(char *p) {\n    return *p + *(p + 1) + *(p + 2);\n}\n\nint main() {\n    char buf[3] = {1, 2, 3};\n    return sum3(buf);\n}\n",
        6,
    );
    run_qemu_case(
        "int-array-initializer-list",
        "int sum3(int *p) {\n    return *p + *(p + 1) + *(p + 2);\n}\n\nint main() {\n    int nums[3] = {10, 20, 30};\n    return sum3(nums);\n}\n",
        60,
    );
    run_qemu_case(
        "empty-char-array-initializer-list-zero-fills",
        "int sum3(char *p) {\n    return *p + *(p + 1) + *(p + 2);\n}\n\nint main() {\n    char buf[3] = {};\n    return sum3(buf);\n}\n",
        0,
    );
    run_qemu_case(
        "empty-int-array-initializer-list-zero-fills",
        "int sum3(int *p) {\n    return *p + *(p + 1) + *(p + 2);\n}\n\nint main() {\n    int nums[3] = {};\n    return sum3(nums);\n}\n",
        0,
    );
}

#[test]
fn qemu_compound_assignment_programs_return_expected_values() {
    run_qemu_case(
        "compound-add",
        "int main() {\n    int x = 3;\n    x += 4;\n    return x;\n}\n",
        7,
    );
    run_qemu_case(
        "compound-subtract",
        "int main() {\n    int x = 10;\n    x -= 3;\n    return x;\n}\n",
        7,
    );
    run_qemu_case(
        "compound-multiply",
        "int main() {\n    int x = 3;\n    x *= 4;\n    return x;\n}\n",
        12,
    );
    run_qemu_case(
        "compound-divide",
        "int main() {\n    int x = 8;\n    x /= 2;\n    return x;\n}\n",
        4,
    );
    run_qemu_case(
        "compound-remainder",
        "int main() {\n    int x = 8;\n    x %= 3;\n    return x;\n}\n",
        2,
    );
    run_qemu_case(
        "compound-bit-and",
        "int main() {\n    int x = 6;\n    x &= 3;\n    return x;\n}\n",
        2,
    );
    run_qemu_case(
        "compound-bit-or",
        "int main() {\n    int x = 4;\n    x |= 1;\n    return x;\n}\n",
        5,
    );
    run_qemu_case(
        "compound-bit-xor",
        "int main() {\n    int x = 6;\n    x ^= 3;\n    return x;\n}\n",
        5,
    );
    run_qemu_case(
        "compound-shift-left",
        "int main() {\n    int x = 3;\n    x <<= 2;\n    return x;\n}\n",
        12,
    );
    run_qemu_case(
        "compound-shift-right",
        "int main() {\n    int x = 16;\n    x >>= 2;\n    return x;\n}\n",
        4,
    );
    run_qemu_case(
        "compound-expression-result",
        "int main() {\n    int x = 3;\n    return x += 4;\n}\n",
        7,
    );
    run_qemu_case(
        "compound-char-narrows",
        "int main() {\n    char c = 250;\n    c += 10;\n    return c;\n}\n",
        4,
    );
}

#[test]
fn qemu_increment_decrement_programs_return_expected_values() {
    run_qemu_case(
        "prefix-increment-result",
        "int main() {\n    int x = 1;\n    return ++x;\n}\n",
        2,
    );
    run_qemu_case(
        "postfix-increment-result",
        "int main() {\n    int x = 1;\n    return x++;\n}\n",
        1,
    );
    run_qemu_case(
        "postfix-increment-side-effect",
        "int main() {\n    int x = 1;\n    x++;\n    return x;\n}\n",
        2,
    );
    run_qemu_case(
        "prefix-decrement-result",
        "int main() {\n    int x = 1;\n    return --x;\n}\n",
        0,
    );
    run_qemu_case(
        "postfix-decrement-result",
        "int main() {\n    int x = 1;\n    return x--;\n}\n",
        1,
    );
    run_qemu_case(
        "char-prefix-increment-narrows",
        "int main() {\n    char c = 255;\n    ++c;\n    return c;\n}\n",
        0,
    );
    run_qemu_case(
        "char-postfix-increment-result",
        "int main() {\n    char c = 255;\n    return c++;\n}\n",
        255,
    );
}

#[test]
fn qemu_char_narrowing_programs_return_expected_values() {
    run_qemu_case(
        "char-local-initializer-narrows",
        "int main() {\n    char c = 300;\n    return c;\n}\n",
        44,
    );
    run_qemu_case(
        "char-assignment-narrows",
        "int main() {\n    char c;\n    c = 300;\n    return c;\n}\n",
        44,
    );
    run_qemu_case(
        "char-assignment-expression-result-narrows",
        "int main() {\n    char c;\n    int x = c = 300;\n    return x == 44;\n}\n",
        1,
    );
    run_qemu_case(
        "char-return-narrows",
        "char main() {\n    return 300;\n}\n",
        44,
    );
    run_qemu_case(
        "char-parameter-narrows",
        "int id(char x) {\n    return x;\n}\n\nint main() {\n    return id(300);\n}\n",
        44,
    );
}

#[test]
fn qemu_char_literal_programs_return_expected_values() {
    run_qemu_case(
        "char-literal-return",
        "int main() {\n    return 'A';\n}\n",
        65,
    );
    run_qemu_case(
        "char-literal-in-char-local",
        "int main() {\n    char c = 'A';\n    return c;\n}\n",
        65,
    );
    run_qemu_case(
        "escaped-newline-char-literal",
        "int main() {\n    char c = '\\n';\n    return c;\n}\n",
        10,
    );
    run_qemu_case(
        "escaped-quote-char-literal",
        "int main() {\n    return '\\'';\n}\n",
        39,
    );
}

#[test]
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
        "mixed-declarations-and-statements",
        "int main() {\n    int x = 1;\n    x = x + 1;\n    int y = x + 2;\n    return y;\n}\n",
        4,
    );
    run_qemu_case(
        "nested-blocks",
        "int main() {\n    int x = 1;\n    {\n        int y = 2;\n        {\n            int z = 3;\n            return x + y + z;\n        }\n    }\n}\n",
        6,
    );
}

#[test]
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
    run_qemu_case(
        "do-while-runs-body-before-condition",
        "int main() {\n    int x = 0;\n    do {\n        x = x + 1;\n    } while (0);\n    return x;\n}\n",
        1,
    );
    run_qemu_case(
        "do-while-countdown",
        "int main() {\n    int x = 3;\n    do {\n        x = x - 1;\n    } while (x);\n    return x;\n}\n",
        0,
    );
    run_qemu_case(
        "continue-in-do-while-runs-condition",
        "int main() {\n    int x = 0;\n    int sum = 0;\n    do {\n        x = x + 1;\n        if (x == 3) continue;\n        sum = sum + x;\n    } while (x < 5);\n    return sum;\n}\n",
        12,
    );
}

#[test]
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
        "prototype-before-forward-call",
        "int helper(int);\n\nint main() {\n    return helper(4);\n}\n\nint helper(int value) {\n    return value + 3;\n}\n",
        7,
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
    run_qemu_case(
        "void-function-bare-return",
        "void set(int *p) {\n    *p = 7;\n    return;\n}\n\nint main() {\n    int x = 0;\n    set(&x);\n    return x;\n}\n",
        7,
    );
    run_qemu_case(
        "void-parameter-list",
        "int helper(void);\n\nint helper(void) {\n    return 3;\n}\n\nint main(void) {\n    return helper();\n}\n",
        3,
    );
    run_qemu_case(
        "static-helper-function",
        "static int helper(void) {\n    return 11;\n}\n\nint main(void) {\n    return helper();\n}\n",
        11,
    );
    run_qemu_case(
        "pointer-return-function-call",
        "int *id(int *);\n\nint *id(int *p) {\n    return p;\n}\n\nint main() {\n    int *p;\n    id(p);\n    return 7;\n}\n",
        7,
    );
}

#[test]
fn qemu_function_pointer_programs_return_expected_values() {
    run_qemu_case(
        "function-pointer-indirect-call",
        "int id(int x) {\n    return x + 1;\n}\n\nint main() {\n    int (*fp)(int) = id;\n    return fp(3);\n}\n",
        4,
    );
    run_qemu_case(
        "explicitly-dereferenced-function-pointer-indirect-call",
        "int id(int x) {\n    return x + 1;\n}\n\nint main() {\n    int (*fp)(int) = id;\n    return (*fp)(3);\n}\n",
        4,
    );
    run_qemu_case(
        "function-pointer-call-with-two-arguments",
        "int add(int x, int y) {\n    return x + y;\n}\n\nint main() {\n    int (*fp)(int, int) = add;\n    return fp(2, 5);\n}\n",
        7,
    );
    run_qemu_case(
        "typedef-function-pointer-indirect-call",
        "typedef int (*handler)(int);\n\nint twice(int x) {\n    return x * 2;\n}\n\nint main() {\n    handler fp = twice;\n    return fp(6);\n}\n",
        12,
    );
    run_qemu_case(
        "function-designator-decays-in-call-argument",
        "int id(int x) {\n    return x;\n}\n\nint apply(int (*f)(int), int x) {\n    return f(x);\n}\n\nint main() {\n    return apply(id, 3);\n}\n",
        3,
    );
    run_qemu_case(
        "function-designator-decays-in-pointer-comparison",
        "int id(int x) {\n    return x;\n}\n\nint main() {\n    int (*fp)(int) = id;\n    return fp == id;\n}\n",
        1,
    );
    run_qemu_case(
        "explicitly-dereferenced-function-designator-call",
        "int id(int x) {\n    return x + 1;\n}\n\nint main() {\n    return (*id)(3);\n}\n",
        4,
    );
}

#[test]
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

#[test]
fn qemu_array_indexing_programs_return_expected_values() {
    run_qemu_case(
        "char-array-index-constant-rvalue",
        "int main() {\n    char buf[3] = {'a', 'b', 'c'};\n    return buf[0];\n}\n",
        97,
    );

    run_qemu_case(
        "char-array-index-variable-rvalue",
        "int main() {\n    char buf[3] = {'a', 'b', 'c'};\n    int i = 1;\n    return buf[i];\n}\n",
        98,
    );

    run_qemu_case(
        "int-array-index-constant-rvalue",
        "int main() {\n    int nums[3] = {10, 20, 30};\n    return nums[2];\n}\n",
        30,
    );

    run_qemu_case(
        "int-array-index-variable-rvalue",
        "int main() {\n    int nums[3] = {10, 20, 30};\n    int i = 1;\n    return nums[i];\n}\n",
        20,
    );

    run_qemu_case(
        "char-array-index-assignment-lvalue",
        "int main() {\n    char buf[3] = {'a', 'b', 'c'};\n    buf[1] = 'x';\n    return buf[1];\n}\n",
        120,
    );

    run_qemu_case(
        "int-array-index-assignment-lvalue",
        "int main() {\n    int nums[3] = {10, 20, 30};\n    nums[1] = 77;\n    return nums[1];\n}\n",
        77,
    );

    run_qemu_case(
        "char-array-index-compound-assignment",
        "int main() {\n    char buf[3] = {1, 2, 3};\n    buf[1] += 40;\n    return buf[1];\n}\n",
        42,
    );

    run_qemu_case(
        "int-array-index-postfix-increment",
        "int main() {\n    int nums[3] = {10, 20, 30};\n    int old = nums[1]++;\n    return old + nums[1];\n}\n",
        41,
    );

    run_qemu_case(
        "pointer-index-rvalue",
        "int main() {\n    char buf[3] = {'a', 'b', 'c'};\n    char *p = buf;\n    return p[2];\n}\n",
        99,
    );

    run_qemu_case(
        "pointer-index-assignment-lvalue",
        "int main() {\n    int nums[3] = {10, 20, 30};\n    int *p = nums;\n    p[2] = 44;\n    return nums[2];\n}\n",
        44,
    );

    run_qemu_case(
        "array-index-expression",
        "int main() {\n    int nums[4] = {10, 20, 30, 40};\n    int i = 1;\n    return nums[i + 2];\n}\n",
        40,
    );

    run_qemu_case(
        "array-index-in-loop-sum",
        "int main() {\n    char buf[3] = {1, 2, 3};\n    int sum = 0;\n    for (int i = 0; i < 3; i = i + 1) {\n        sum += buf[i];\n    }\n    return sum;\n}\n",
        6,
    );

    run_qemu_case(
        "array-index-zero-filled-tail",
        "int main() {\n    int nums[4] = {7};\n    return nums[0] + nums[1] + nums[2] + nums[3];\n}\n",
        7,
    );

    run_qemu_case(
        "global-int-array-index-zero-filled-tail",
        "int nums[4] = {7, 8};\n\nint main() {\n    return nums[0] + nums[1] + nums[2] + nums[3];\n}\n",
        15,
    );

    run_qemu_case(
        "global-char-array-index-rvalue",
        "char buf[3] = {'a', 'b', 'c'};\n\nint main() {\n    return buf[1];\n}\n",
        98,
    );

    run_qemu_case(
        "static-global-int-rvalue",
        "static int g = 17;\n\nint main(void) {\n    return g;\n}\n",
        17,
    );

    run_qemu_case(
        "array-initializer-allows-trailing-comma",
        "int main() {\n    unsigned char buf[3] = {1, 2, 3,};\n    return buf[0] + buf[1] + buf[2];\n}\n",
        6,
    );

    run_qemu_case(
        "pointer-index-after-pointer-arithmetic",
        "int main() {\n    char buf[4] = {'a', 'b', 'c', 'd'};\n    char *p = buf + 1;\n    return p[1];\n}\n",
        99,
    );

    run_qemu_case(
        "array-index-prefix-increment",
        "int main() {\n    int nums[3] = {10, 20, 30};\n    return ++nums[1];\n}\n",
        21,
    );
}

#[test]
fn qemu_postfix_pointer_dereference_programs_return_expected_values() {
    run_qemu_case(
        "address-of-local-through-pointer",
        "int main() {\n    int x = 0;\n    int *p = &x;\n    *p = 7;\n    return x;\n}\n",
        7,
    );

    run_qemu_case(
        "address-of-global-through-pointer",
        "int g;\n\nint main() {\n    int *p = &g;\n    *p = 5;\n    return g;\n}\n",
        5,
    );

    run_qemu_case(
        "parenthesized-pointer-to-array-index",
        "int main() {\n    int arr[3] = {1, 2, 3};\n    int (*p)[3] = &arr;\n    return (*p)[1];\n}\n",
        2,
    );

    run_qemu_case(
        "function-pointer-local-declaration",
        "int main() {\n    int (*fp)(int, char *);\n    return 0;\n}\n",
        0,
    );

    run_qemu_case(
        "function-pointer-initialized-from-function-designator",
        "int id(int x) {\n    return x;\n}\n\nint main() {\n    int (*fp)(int) = id;\n    return 0;\n}\n",
        0,
    );

    run_qemu_case(
        "postfix-pointer-dereference",
        "int main() {\n    char buf[3] = {'a', 'b', 'c'};\n    char *p = buf;\n    return *p++;\n}\n",
        97,
    );

    run_qemu_case(
        "postfix-pointer-dereference-side-effect",
        "int main() {\n    char buf[3] = {'a', 'b', 'c'};\n    char *p = buf;\n    *p++;\n    return *p;\n}\n",
        98,
    );

    run_qemu_case(
        "postfix-pointer-dereference-old-value",
        "int main() {\n    char buf[3] = {'a', 'b', 'c'};\n    char *p = buf;\n    int a = *p++;\n    int b = *p++;\n    return a + b;\n}\n",
        195,
    );
}

#[test]
fn qemu_null_pointer_constant_programs_return_expected_values() {
    run_qemu_case(
        "pointer-equals-null-right",
        "int main() {\n    char *p = 0;\n    return p == 0;\n}\n",
        1,
    );

    run_qemu_case(
        "null-equals-pointer-left",
        "int main() {\n    char *p = 0;\n    return 0 == p;\n}\n",
        1,
    );

    run_qemu_case(
        "pointer-not-equals-null-false",
        "int main() {\n    char *p = 0;\n    return p != 0;\n}\n",
        0,
    );

    run_qemu_case(
        "non-null-pointer-not-equals-null",
        "int main() {\n    char buf[1] = {'a'};\n    char *p = buf;\n    return p != 0;\n}\n",
        1,
    );

    run_qemu_case(
        "non-null-pointer-equals-null-false",
        "int main() {\n    char buf[1] = {'a'};\n    char *p = buf;\n    return p == 0;\n}\n",
        0,
    );

    run_qemu_case(
        "assign-null-to-pointer",
        "int main() {\n    char buf[1] = {'a'};\n    char *p = buf;\n    p = 0;\n    return p == 0;\n}\n",
        1,
    );

    run_qemu_case(
        "pass-null-to-pointer-parameter",
        "int is_null(char *p) {\n    return p == 0;\n}\n\nint main() {\n    return is_null(0);\n}\n",
        1,
    );

    run_qemu_case(
        "return-null-from-pointer-function",
        "char *null_ptr() {\n    return 0;\n}\n\nint main() {\n    return null_ptr() == 0;\n}\n",
        1,
    );

    run_qemu_case(
        "array-of-pointers-null-initializers",
        "int main() {\n    char *ptrs[2] = {0, 0};\n    return ptrs[0] == 0 && ptrs[1] == 0;\n}\n",
        1,
    );

    run_qemu_case(
        "array-of-pointers-partial-null-initializer",
        "int main() {\n    char buf[1] = {'a'};\n    char *ptrs[2] = {buf};\n    return ptrs[0] != 0 && ptrs[1] == 0;\n}\n",
        1,
    );

    run_qemu_case(
        "compatible-pointers-compare-equal",
        "int main() {\n    char buf[1] = {'a'};\n    char *p = buf;\n    char *q = buf;\n    return p == q;\n}\n",
        1,
    );

    run_qemu_case(
        "compatible-pointers-compare-not-equal",
        "int main() {\n    char buf[2] = {'a', 'b'};\n    char *p = buf;\n    char *q = buf + 1;\n    return p != q;\n}\n",
        1,
    );

    run_qemu_case(
        "zero-valued-constant-expression-initializes-pointer",
        "int main() {\n    char *p = 0 + 0;\n    return p == 0;\n}\n",
        1,
    );

    run_qemu_case(
        "zero-valued-constant-expression-compares-with-pointer",
        "int main() {\n    char *p = 0;\n    return p == 0 * 1;\n}\n",
        1,
    );
}

#[test]
fn qemu_void_pointer_programs_return_expected_values() {
    run_qemu_case(
        "void-pointer-from-char-pointer",
        "int main() {\n    char *s = \"abc\";\n    void *p = s;\n    return p != 0;\n}\n",
        1,
    );

    run_qemu_case(
        "char-pointer-from-void-pointer",
        "int main() {\n    char *s = \"abc\";\n    void *p = s;\n    char *q = p;\n    return q[1];\n}\n",
        98,
    );

    run_qemu_case(
        "void-pointer-parameter-from-char-pointer",
        "int has_pointer(void *p) {\n    return p != 0;\n}\nint main() {\n    char *s = \"abc\";\n    return has_pointer(s);\n}\n",
        1,
    );

    run_qemu_case(
        "char-pointer-parameter-from-void-pointer",
        "int second(char *p) {\n    return p[1];\n}\nint main() {\n    char *s = \"abc\";\n    void *p = s;\n    return second(p);\n}\n",
        98,
    );

    run_qemu_case(
        "void-pointer-compares-equal-to-char-pointer",
        "int main() {\n    char *s = \"abc\";\n    void *p = s;\n    return p == s;\n}\n",
        1,
    );
}
