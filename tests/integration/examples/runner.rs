use crate::common::EXAMPLES_DIR;
use std::process::Command;

pub fn run_example(path: &str) -> String {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/{}", EXAMPLES_DIR, path);
    let mut cmd = Command::new(binary);
    cmd.current_dir(manifest_dir).arg(&full_path);

    let output = cmd.output().expect("failed to execute example");
    assert!(
        output.status.success(),
        "example {} exited with status {:?}\nstdout:\n{}\nstderr:\n{}",
        path,
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("stdout was not utf8")
}
