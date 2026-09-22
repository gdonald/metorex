// What the command line does with the switches and the program it is
// given.

pub(crate) use std::process::Command;

pub(crate) fn metorex_cmd() -> Command {
    let binary = env!("CARGO_BIN_EXE_metorex");
    Command::new(binary)
}

// -K, -E, -U and the long encoding flags

/// Run one of the `cli_flags/show_encodings` examples under the given flags.
pub(crate) fn show_encodings(flags: &[&str], script: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/tests/_examples/cli_flags/{}", manifest_dir, script);
    let mut arguments: Vec<&str> = flags.to_vec();
    arguments.push(&path);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(arguments)
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Run one of the `cli_flags` examples with the flags given, answering what it
/// wrote to standard output.
pub(crate) fn run_cli_flags_example(flags: &[&str], script: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/tests/_examples/cli_flags/{}", manifest_dir, script);
    let mut arguments: Vec<&str> = flags.to_vec();
    arguments.push(&path);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(arguments)
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Run one of the `cli_flags` examples with the flags given, answering what it
/// wrote to standard error.
pub(crate) fn cli_flags_example_stderr(flags: &[&str], script: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("tests/_examples/cli_flags/{}", script);
    let mut arguments: Vec<&str> = flags.to_vec();
    arguments.push(&path);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(arguments)
        .output()
        .expect("failed to execute");
    String::from_utf8(output.stderr).unwrap()
}

mod errors;
mod flags;
mod running;
