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

/// The expected output of both `filesystem/pipe_streams` variants.
const PIPE_STREAMS_OUTPUT: &str = "true\ntrue\nIO\nfalse\n1\nover the pipe\ntrue\ntrue\ntrue\n";

#[test]
fn test_filesystem_pipe_streams_execution() {
    let output = run_example("filesystem/pipe_streams.rb");
    assert_eq!(output, PIPE_STREAMS_OUTPUT);
}

#[test]
fn test_filesystem_pipe_streams_no_parens_execution() {
    let output = run_example("filesystem/pipe_streams_no_parens.rb");
    assert_eq!(output, PIPE_STREAMS_OUTPUT);
}

/// The expected output of both `filesystem/stream_redirection` variants.
const STREAM_REDIRECTION_OUTPUT: &str = "0\n1\n\"from the first\\n\"\n\"metorex_stream_second.txt\"\n\"from the second\\n\"\n1\n2\n3\n\ntrue\n";

#[test]
fn test_filesystem_stream_redirection_execution() {
    let output = run_example("filesystem/stream_redirection.rb");
    assert_eq!(output, STREAM_REDIRECTION_OUTPUT);
}

#[test]
fn test_filesystem_stream_redirection_no_parens_execution() {
    let output = run_example("filesystem/stream_redirection_no_parens.rb");
    assert_eq!(output, STREAM_REDIRECTION_OUTPUT);
}

/// The expected output of both `filesystem/stream_bytes` variants.
const STREAM_BYTES_OUTPUT: &str =
    "97\n\"b\"\n122\n\"c\"\nnil\n[97, 98, 99]\n[\"a\", \"b\", \"c\"]\nnil\n\"ab\"\n0\n\"held\"\n";

#[test]
fn test_filesystem_stream_bytes_execution() {
    let output = run_example("filesystem/stream_bytes.rb");
    assert_eq!(output, STREAM_BYTES_OUTPUT);
}

#[test]
fn test_filesystem_stream_bytes_parens_execution() {
    let output = run_example("filesystem/stream_bytes_parens.rb");
    assert_eq!(output, STREAM_BYTES_OUTPUT);
}

/// The expected output of both `filesystem/descriptor_streams` variants, which differ only in whether
/// the calls are written with parentheses.
const DESCRIPTOR_STREAMS_OUTPUT: &str = "true\n14\nnil\n[\"UTF-8\", \"ISO-8859-1\"]\n\"one\\n\"\n14\n\"one\\n\"\n4\n0\n\"three\\n\"\n0\n14\n\"one\"\n\"two\"\n\"three\"\n[\"one\", \"two\", \"three\"]\n[\"one\", \"two\", \"three\"]\n3\n\"one\"\n";

#[test]
fn test_filesystem_descriptor_streams_execution() {
    let output = run_example("filesystem/descriptor_streams.rb");
    assert_eq!(output, DESCRIPTOR_STREAMS_OUTPUT);
}

#[test]
fn test_filesystem_descriptor_streams_parens_execution() {
    let output = run_example("filesystem/descriptor_streams_parens.rb");
    assert_eq!(output, DESCRIPTOR_STREAMS_OUTPUT);
}

/// The expected output of both `filesystem/line_reading` variants, which differ only in whether
/// the calls are written with parentheses.
const LINE_READING_OUTPUT: &str = "\"one two\\n\"\n\"three \"\n\"four\"\n\"\"\n4\n\"one two\\nthree four\\n\\n\"\n\"five six\\n\"\n\"one two\"\n[\"three four\", \"\", \"five six\"]\n[\"one two\", \"three four\", \"\", \"five six\"]\n\"one\"\n\" t\"\n\"sysread for buffered IO\"\n\"one\"\n\" two\"\n";

#[test]
fn test_filesystem_line_reading_execution() {
    let output = run_example("filesystem/line_reading.rb");
    assert_eq!(output, LINE_READING_OUTPUT);
}

#[test]
fn test_filesystem_line_reading_parens_execution() {
    let output = run_example("filesystem/line_reading_parens.rb");
    assert_eq!(output, LINE_READING_OUTPUT);
}

/// The expected output of both `filesystem/argf_streams` variants, which differ only in whether
/// the calls are written with parentheses.
const ARGF_STREAMS_OUTPUT: &str = "[111, 110, 101, 10]\n[\"o\", \"n\", \"e\", \"\\n\", \"t\", \"w\", \"o\", \"\\n\"]\n8\n[\"one\\n\", \"two\\n\"]\n\"one\\n\"\ntrue\n\"two\\n\"\nnil\n\"one\\n\"\n\"one\\n\"\n\"\"\n\"two\\n\"\n\"US-ASCII\"\n\"US-ASCII\"\n";

#[test]
fn test_filesystem_argf_streams_execution() {
    let output = run_example("filesystem/argf_streams.rb");
    assert_eq!(output, ARGF_STREAMS_OUTPUT);
}

#[test]
fn test_filesystem_argf_streams_parens_execution() {
    let output = run_example("filesystem/argf_streams_parens.rb");
    assert_eq!(output, ARGF_STREAMS_OUTPUT);
}

/// The expected output of both `filesystem/paths_and_locks` variants, which differ only in whether the
/// calls are written with parentheses.
const PATHS_AND_LOCKS_OUTPUT: &str = concat!(
    "/home/jason\n",
    "/home\n",
    "/foo\n",
    ".\n",
    "/foo/..\n",
    "0\n",
    "0\n",
);

#[test]
fn test_filesystem_paths_and_locks_execution() {
    let output = run_example("filesystem/paths_and_locks.rb");
    assert_eq!(output, PATHS_AND_LOCKS_OUTPUT);
}

#[test]
fn test_filesystem_paths_and_locks_no_parens_execution() {
    let output = run_example("filesystem/paths_and_locks_no_parens.rb");
    assert_eq!(output, PATHS_AND_LOCKS_OUTPUT);
}

/// The expected output of both `filesystem/working_directory` variants, which
/// differ only in whether the calls are written with parentheses.
const WORKING_DIRECTORY_OUTPUT: &str = "true\ntrue\n0\n\"inner\"\n\"inner\"\nErrno::ENOENT\n";

#[test]
fn test_filesystem_working_directory_execution() {
    let output = run_example("filesystem/working_directory.rb");
    assert_eq!(output, WORKING_DIRECTORY_OUTPUT);
}

#[test]
fn test_filesystem_working_directory_parens_execution() {
    let output = run_example("filesystem/working_directory_parens.rb");
    assert_eq!(output, WORKING_DIRECTORY_OUTPUT);
}

/// The expected output of both `filesystem/permission_bits` variants, which
/// differ only in whether the calls are written with parentheses.
const PERMISSION_BITS_OUTPUT: &str = "1\n\"444\"\n0\n\"600\"\nErrno::ENOENT\nRangeError\n";

#[test]
fn test_filesystem_permission_bits_execution() {
    let output = run_example("filesystem/permission_bits.rb");
    assert_eq!(output, PERMISSION_BITS_OUTPUT);
}

#[test]
fn test_filesystem_permission_bits_parens_execution() {
    let output = run_example("filesystem/permission_bits_parens.rb");
    assert_eq!(output, PERMISSION_BITS_OUTPUT);
}

/// The expected output of both `filesystem/glob_matching` variants, which
/// differ only in whether the calls are written with parentheses.
const GLOB_MATCHING_OUTPUT: &str = concat!(
    "true\ntrue\nfalse\ntrue\nfalse\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\n",
    "true\nfalse\n"
);

#[test]
fn test_filesystem_glob_matching_execution() {
    let output = run_example("filesystem/glob_matching.rb");
    assert_eq!(output, GLOB_MATCHING_OUTPUT);
}

#[test]
fn test_filesystem_glob_matching_parens_execution() {
    let output = run_example("filesystem/glob_matching_parens.rb");
    assert_eq!(output, GLOB_MATCHING_OUTPUT);
}

/// The expected output of both `filesystem/reopened_streams` variants, which
/// differ only in whether the calls are written with parentheses.
const REOPENED_STREAMS_OUTPUT: &str = "\"first line\\n\"\ntrue\ntrue\n\"second line\\n\"\ntrue\n\"written through\"\nFile\nIO\n\"back again\\n\"\ntrue\nfalse\ntrue\n";

#[test]
fn test_filesystem_reopened_streams_execution() {
    let output = run_example("filesystem/reopened_streams.rb");
    assert_eq!(output, REOPENED_STREAMS_OUTPUT);
}

#[test]
fn test_filesystem_reopened_streams_parens_execution() {
    let output = run_example("filesystem/reopened_streams_parens.rb");
    assert_eq!(output, REOPENED_STREAMS_OUTPUT);
}

/// The expected output of both `filesystem/name_matching` variants, which differ only in
/// whether the calls are written with parentheses.
const NAME_MATCHING_OUTPUT: &str = "[\"a/b\", \"a/one.rb\", \"a/two.txt\"]\n[\"a/one.rb\"]\n[\"a/one.rb\", \"a/two.txt\"]\n[\"a/one.rb\"]\n[\"a/one.rb\"]\n[\"a/b/c/four.rb\", \"a/b/three.rb\", \"a/one.rb\"]\n[\"a/.\", \"a/.hidden\"]\n[\".\", \"a\"]\n[\"/\", \"b/\", \"b/c/\"]\n[\"c\", \"three.rb\"]\n[\"a/one.rb\"]\n[]\n[]\nnil\n[\"a/one.rb\"]\n";

#[test]
fn test_filesystem_name_matching_execution() {
    let output = run_example("filesystem/name_matching.rb");
    assert_eq!(output, NAME_MATCHING_OUTPUT);
}

#[test]
fn test_filesystem_name_matching_parens_execution() {
    let output = run_example("filesystem/name_matching_parens.rb");
    assert_eq!(output, NAME_MATCHING_OUTPUT);
}

/// The expected output of both `filesystem/waiting_pipes` variants, which
/// differ only in whether the calls are written with parentheses.
const WAITING_PIPES_OUTPUT: &str = concat!(
    "\"hello\"\n",
    "true\n",
    "#<Encoding:UTF-16BE>\n",
    "#<Encoding:UTF-8>\n",
    "IOError\n"
);

#[test]
fn test_filesystem_waiting_pipes_execution() {
    let output = run_example("filesystem/waiting_pipes.rb");
    assert_eq!(output, WAITING_PIPES_OUTPUT);
}

#[test]
fn test_filesystem_waiting_pipes_parens_execution() {
    let output = run_example("filesystem/waiting_pipes_parens.rb");
    assert_eq!(output, WAITING_PIPES_OUTPUT);
}

/// The expected output of both `filesystem/written_encodings` variants, which differ only in whether the
/// calls are written with parentheses.
const WRITTEN_ENCODINGS_OUTPUT: &str = concat!(
    "[108, 195, 173, 110, 101, 97, 10]\n",
    "\"línea\\n\"\n",
    "[195, 169]\n",
    "\"uno·\"\n",
    "\"dos·tres\"\n",
);

#[test]
fn test_filesystem_written_encodings_execution() {
    let output = run_example("filesystem/written_encodings.rb");
    assert_eq!(output, WRITTEN_ENCODINGS_OUTPUT);
}

#[test]
fn test_filesystem_written_encodings_parens_execution() {
    let output = run_example("filesystem/written_encodings_parens.rb");
    assert_eq!(output, WRITTEN_ENCODINGS_OUTPUT);
}
