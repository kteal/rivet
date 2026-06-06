use std::fs;
use std::process::Command;

fn run_diagnostic_case(name: &str, source: &str) -> String {
    let tempdir = tempfile::tempdir().expect("failed to create temporary directory");
    let path = tempdir.path().join(format!("{name}.c"));

    fs::write(&path, source).expect("failed to write temporary source file");

    let output = Command::new(env!("CARGO_BIN_EXE_rivet"))
        .arg(&path)
        .output()
        .expect("failed to run compiler");

    assert!(
        !output.status.success(),
        "compiler unexpectedly succeeded for {name}"
    );

    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_diagnostic_contains(name: &str, source: &str, expected_stderr: &str) {
    let stderr = run_diagnostic_case(name, source);

    assert!(
        stderr.contains(expected_stderr),
        "expected stderr for {name} to contain {expected_stderr:?}, got {stderr:?}"
    );
}

#[test]
fn lex_errors_include_filename_line_and_column() {
    let stderr = run_diagnostic_case("lex-error-location", "int main() {\n    return @;\n}\n");

    assert!(
        stderr.contains("lex-error-location.c:2:12: error:"),
        "stderr should include filename, line, column, and severity, got {stderr:?}"
    );
    assert!(
        stderr.contains("unexpected character '@'"),
        "stderr should explain the lexing problem, got {stderr:?}"
    );
}

#[test]
fn parse_errors_include_filename_line_and_column() {
    let stderr = run_diagnostic_case(
        "parse-error-location",
        "int main() {\n    return (1 + );\n}\n",
    );

    assert!(
        stderr.contains("parse-error-location.c:2:17: error:"),
        "stderr should point at the token that made parsing fail, got {stderr:?}"
    );
    assert!(
        stderr.contains("expected expression"),
        "stderr should explain the parsing problem, got {stderr:?}"
    );
}

#[test]
fn semantic_errors_include_filename_line_and_column() {
    let stderr = run_diagnostic_case(
        "semantic-error-location",
        "int main() {\n    return x;\n}\n",
    );

    assert!(
        stderr.contains("semantic-error-location.c:2:12: error:"),
        "stderr should point at the undeclared identifier, got {stderr:?}"
    );
    assert!(
        stderr.contains("undeclared local variable 'x'"),
        "stderr should explain the semantic problem, got {stderr:?}"
    );
}

#[test]
fn duplicate_parameter_errors_point_at_duplicate_parameter() {
    let stderr = run_diagnostic_case(
        "semantic-duplicate-parameter-location",
        "int main(int x, int x) {\n    return x;\n}\n",
    );

    assert!(
        stderr.contains("semantic-duplicate-parameter-location.c:1:21: error:"),
        "stderr should point at the duplicate parameter, got {stderr:?}"
    );
    assert!(
        stderr.contains("duplicate local variable 'x'"),
        "stderr should explain the semantic problem, got {stderr:?}"
    );
}

#[test]
fn semantic_errors_are_reported_before_codegen() {
    assert_diagnostic_contains(
        "semantic-undeclared-return",
        "int main() {\n    return x;\n}\n",
        "error: undeclared local variable 'x'",
    );
    assert_diagnostic_contains(
        "semantic-undeclared-assignment",
        "int main() {\n    x = 1;\n    return 0;\n}\n",
        "error: undeclared local variable 'x'",
    );
    assert_diagnostic_contains(
        "semantic-duplicate-local",
        "int main() {\n    int x = 1;\n    int x = 2;\n    return x;\n}\n",
        "error: duplicate local variable 'x'",
    );
    assert_diagnostic_contains(
        "semantic-later-local",
        "int main() {\n    int y = x;\n    int x = 1;\n    return y;\n}\n",
        "error: undeclared local variable 'x'",
    );
    assert_diagnostic_contains(
        "semantic-block-local-after-scope",
        "int main() {\n    {\n        int x = 1;\n    }\n    return x;\n}\n",
        "error: undeclared local variable 'x'",
    );
    assert_diagnostic_contains(
        "semantic-undeclared-function-call",
        "int main() {\n    return helper();\n}\n",
        "error: undeclared function 'helper'",
    );
    assert_diagnostic_contains(
        "semantic-break-outside-loop",
        "int main() {\n    break;\n}\n",
        "error: cannot use 'break' outside of a loop",
    );
    assert_diagnostic_contains(
        "semantic-continue-outside-loop",
        "int main() {\n    continue;\n}\n",
        "error: cannot use 'continue' outside of a loop",
    );
}
