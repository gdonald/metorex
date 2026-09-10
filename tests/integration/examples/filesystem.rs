// Examples covering File::Stat and the questions File asks about a file

use super::run_example;

/// The expected output of both `filesystem/file_facts` variants, which differ
/// only in whether the calls are written with parentheses.
const FILE_FACTS_OUTPUT: &str = "File::Stat\n\"file\"\ntrue\nfalse\nfalse\n\"644\"\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\nTime\nTime\nfalse\ntrue\nnil\n\"directory\"\nfalse\nfalse\n\"directory\"\ntrue\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\ntrue\nfalse\ntrue\n\"link\"\n\"file\"\n5\ntrue\n2\ntrue\n1200000000\n0\nfalse\n";

#[test]
fn test_filesystem_file_facts_execution() {
    let output = run_example("filesystem/file_facts.rb");
    assert_eq!(output, FILE_FACTS_OUTPUT);
}

#[test]
fn test_filesystem_file_facts_no_parens_execution() {
    let output = run_example("filesystem/file_facts_no_parens.rb");
    assert_eq!(output, FILE_FACTS_OUTPUT);
}

/// The expected output of both `filesystem/walking_a_directory` variants,
/// which differ only in whether the calls are written with parentheses.
const WALKING_A_DIRECTORY_OUTPUT: &str = "Dir\ntrue\ntrue\ntrue\nfalse\ntrue\n0\nString\n1\n0\ntrue\ntrue\n[\".\", \"..\", \"inside\", \"one.txt\", \"two.txt\"]\n[\"inside\", \"one.txt\", \"two.txt\"]\n[\".\", \"..\", \"inside\", \"one.txt\", \"two.txt\"]\nnil\n[\"inside\", \"one.txt\", \"two.txt\"]\nnil\ntrue\ntrue\n\"closed directory\"\n[\".\", \"..\", \"inside\", \"one.txt\", \"two.txt\"]\n[\"inside\", \"one.txt\", \"two.txt\"]\ntrue\n3\nfalse\n";

#[test]
fn test_filesystem_walking_a_directory_execution() {
    let output = run_example("filesystem/walking_a_directory.rb");
    assert_eq!(output, WALKING_A_DIRECTORY_OUTPUT);
}

#[test]
fn test_filesystem_walking_a_directory_no_parens_execution() {
    let output = run_example("filesystem/walking_a_directory_no_parens.rb");
    assert_eq!(output, WALKING_A_DIRECTORY_OUTPUT);
}

/// The expected output of both `filesystem/asking_about_a_file` variants.
const ASKING_ABOUT_A_FILE_OUTPUT: &str = "true\ntrue\ntrue\ntrue\ntrue\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\ntrue\nnil\ntrue\ntrue\ntrue\ntrue\nTime\ntrue\ntrue\n";

#[test]
fn test_filesystem_asking_about_a_file_execution() {
    let output = run_example("filesystem/asking_about_a_file.rb");
    assert_eq!(output, ASKING_ABOUT_A_FILE_OUTPUT);
}

#[test]
fn test_filesystem_asking_about_a_file_no_parens_execution() {
    let output = run_example("filesystem/asking_about_a_file_no_parens.rb");
    assert_eq!(output, ASKING_ABOUT_A_FILE_OUTPUT);
}

/// The expected output of both `filesystem/stream_paths/coercion` variants.
const STREAM_PATHS_OUTPUT: &str = "true\ntrue\nfalse\ntrue\n\"held\"\n";

#[test]
fn test_filesystem_stream_paths_coercion_execution() {
    let output = run_example("filesystem/stream_paths/coercion.rb");
    assert_eq!(output, STREAM_PATHS_OUTPUT);
}

#[test]
fn test_filesystem_stream_paths_coercion_parens_execution() {
    let output = run_example("filesystem/stream_paths/coercion_parens.rb");
    assert_eq!(output, STREAM_PATHS_OUTPUT);
}
