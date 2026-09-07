// Examples covering the libraries metorex carries

use super::run_example;

/// The expected output of both `stdlib_libraries/shipped_libraries` variants,
/// which differ only in whether the calls are written with parentheses.
const SHIPPED_LIBRARIES_OUTPUT: &str = "\"Tm93IGlzIHRoZSB0aW1lIGZvciBhbGwgZ29vZCBjb2RlcnMKdG8gbGVhcm4g\\nUnVieQ==\\n\"\n\"aGVsbG8=\"\n\"hello\"\n\"hello\"\n\"aGVsbG8_\"\n\"aGVsbG8_\"\n\"hello?\"\n[\"ruby\", \"-e\", \"puts 1\", \"--name\", \"value\"]\n\"a\\\\ b\"\n\"ruby a\\\\ b\"\n[\"one\", \"two three\"]\n{\"r\" => \"ruby\", \"ru\" => \"ruby\", \"rub\" => \"ruby\", \"ruby\" => \"ruby\"}\n{\"ca\" => \"car\", \"car\" => \"car\", \"co\" => \"cone\", \"con\" => \"cone\", \"cone\" => \"cone\"}\ntrue\n0\nNoMethodError\nTypeError\n0\n1\n:moved\n0\n32\n8\n\"\"\n36\nInteger\n7\n7\n\"e\"\n1\n\"xam\"\n4\nfalse\nnil\nnil\nfalse\ntrue\ntrue\nfalse\ntrue\n\"first\\n\"\n1\n[\"second\\n\"]\n0\n[\"first\\n\", \"second\\n\"]\n[\"a\", \"b\", \"c\"]\n[97, 98, 99]\n\"a\"\n97\n\"one two!\\nthree\"\n14\n13\n0\n2\n0\n\"exam\"\nIOError\n\"Ada\"\n1843\n\"computing\"\n\"Ada\"\n\"London\"\n{name: \"Ada\", year: 1843, field: \"computing\", city: \"London\"}\n\"#<OpenStruct name=\\\"Ada\\\", year=1843, field=\\\"computing\\\", city=\\\"London\\\">\"\ntrue\nfalse\nnil\ntrue\n\"#<OpenStruct>\"\ntrue\nfalse\n[2, 3, 5, 7, 11]\n[[2, 3], [3, 2], [5, 1]]\n72\n[[-1, 1], [2, 1], [5, 1]]\ntrue\n[[2, 2], [3, 1]]\n20\n2\n3\n2\n";

#[test]
fn test_stdlib_libraries_shipped_libraries_execution() {
    let output = run_example("stdlib_libraries/shipped_libraries.rb");
    assert_eq!(output, SHIPPED_LIBRARIES_OUTPUT);
}

#[test]
fn test_stdlib_libraries_shipped_libraries_no_parens_execution() {
    let output = run_example("stdlib_libraries/shipped_libraries_no_parens.rb");
    assert_eq!(output, SHIPPED_LIBRARIES_OUTPUT);
}
