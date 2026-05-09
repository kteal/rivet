use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_qemu_case(name: &str, source: &str, expected: i32) {
    let mut path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    path.push(format!("rivet-{name}-{unique}.c"));

    fs::write(&path, source).expect("failed to write temporary source file");

    let status = Command::new("scripts/run-rv32.sh")
        .arg("--expect")
        .arg(expected.to_string())
        .arg(&path)
        .status()
        .expect("failed to run scripts/run-rv32.sh");

    fs::remove_file(&path).expect("failed to remove temporary source file");

    assert!(status.success(), "qemu runner failed for {name}");
}

#[test]
#[ignore = "requires qemu-riscv32 and riscv64-linux-gnu binutils"]
fn qemu_arithmetic_programs_return_expected_values() {
    run_qemu_case("return-42", "int main() {\n    return 42;\n}\n", 42);
    run_qemu_case("precedence", "int main() {\n    return 1 + 2 * 3;\n}\n", 7);
    run_qemu_case(
        "parentheses",
        "int main() {\n    return (1 + 2) * 3;\n}\n",
        9,
    );
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
    run_qemu_case("div-rem", "int main() {\n    return 8 / 2 + 8 % 3;\n}\n", 6);
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
