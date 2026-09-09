// Examples covering Pathname

use super::run_example;

/// The expected output of both `paths/naming_files` variants, which differ only
/// in whether the calls are written with parentheses.
const NAMING_FILES_OUTPUT: &str = "\"/usr/local/bin\"\n\"#<Pathname:/usr/local/bin>\"\ntrue\nfalse\nfalse\n\"/usr/local\"\n\"bin\"\n\"/usr/local\"\n\".so\"\ntrue\ntrue\ntrue\n\"/usr/bin/ruby\"\n\"/usr/bin\"\n\"/usr/local/bin\"\n\"/etc\"\n\"foo/bar\"\n\"/usr/share\"\n\"/usr/fish/bin/\"\n\"/a/c/d\"\n\"bin/ls\"\n\"usr\"\n\"../a\"\n\".\"\n[\"usr\", \"local\", \"bin\"]\n[\"/\", \"/usr\", \"/usr/local\"]\n[\"/usr/local\", \"/usr\", \"/\"]\ntrue\ntrue\ntrue\nfalse\ntrue\nPathname\n";

#[test]
fn test_paths_naming_files_execution() {
    let output = run_example("paths/naming_files.rb");
    assert_eq!(output, NAMING_FILES_OUTPUT);
}

#[test]
fn test_paths_naming_files_no_parens_execution() {
    let output = run_example("paths/naming_files_no_parens.rb");
    assert_eq!(output, NAMING_FILES_OUTPUT);
}
