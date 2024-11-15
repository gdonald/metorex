// Examples runner

use std::process::Command;

fn run_example(path: &str) -> String {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut cmd = Command::new(binary);
    cmd.current_dir(manifest_dir).arg(path);

    let output = cmd.output().expect("failed to execute example");
    assert!(
        output.status.success(),
        "example {} exited with status {:?}",
        path,
        output.status
    );

    String::from_utf8(output.stdout).expect("stdout was not utf8")
}

#[test]
fn test_basics_greeting_line_execution() {
    let output = run_example("examples/basics/greeting_line.mx");
    assert_eq!(output, "Hello, Ada!\n");
}

#[test]
fn test_basics_string_methods_execution() {
    let expected = r#"=== Basic String Methods ===
ALICE
alice
Hello, World!
xeroteM
11

=== String Inspection Methods ===
H
i
65
66
"#;

    let output = run_example("examples/basics/string_methods.mx");
    assert_eq!(output, expected.to_string());
}
