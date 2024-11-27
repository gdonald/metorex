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

#[test]
fn test_data_structures_simple_dict_execution() {
    let output = run_example("examples/data-structures/simple_dict.mx");
    // Hash map iteration order is non-deterministic, so check both possible orders
    let valid_output1 = "{bob: 25, alice: 30}\n30\n";
    let valid_output2 = "{alice: 30, bob: 25}\n30\n";
    assert!(
        output == valid_output1 || output == valid_output2,
        "Expected either '{}' or '{}', but got '{}'",
        valid_output1,
        valid_output2,
        output
    );
}

#[test]
fn test_data_structures_dict_access_execution() {
    let output = run_example("examples/data-structures/dict_access.mx");
    assert_eq!(output, "Ada lives in London\n");
}

#[test]
fn test_data_structures_hash_methods_execution() {
    let output = run_example("examples/data-structures/hash_methods.mx");
    // Hash map iteration order is non-deterministic, so check for valid orderings
    let fixed_part = "Has alice?\ntrue\nHas dave?\nfalse\nSize:\n3\n";
    assert!(
        output.contains(fixed_part)
            && output.contains("alice")
            && output.contains("bob")
            && output.contains("charlie")
            && output.contains("30")
            && output.contains("25")
            && output.contains("35"),
        "Expected output to contain all keys, values, and fixed text, but got: {}",
        output
    );
}
