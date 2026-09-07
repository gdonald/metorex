use super::run_example;

/// The expected output of both `symbols_and_ranges/names_and_walks` variants,
/// which differ only in whether the calls are written with parentheses.
const NAMES_AND_WALKS_OUTPUT: &str = ":GLARK\n:glark\n:Hello_world\n:aBc\n:b\n:AA\n\"ruby\"\n\"ruby\"\n:ruby\n4\ntrue\n-1\n:a2b\n\"Symbol\"\n\"àBc\"\n:àbc\n\"Über\"\n4\n\"a b\"\n\"HI\"\n\"A\"\n\"あ\"\n\"ba\"\n\"aaa\"\n\"b0\"\n\"2.0\"\n\"Hello world\"\n\"aBc\"\n1234.5\n1234.5\n0.0\n5.0\n[1, 2, 3, 4, 5]\n[\"A\", \"B\", \"C\", \"D\"]\n[]\n[\"ax\", \"ay\", \"az\", \"ba\", \"bb\", \"bc\", \"bd\"]\n[:A, :B, :C, :D]\n\"1...4\"\n\"1...4\"\n\"1..\"\n\"..3\"\ntrue\ntrue\nfalse\n3\nInfinity\n[\"a\", \"b\", \"c\"]\ncannot convert endless range to an array\n[1, 2, 3]\n[1, 2]\n";

#[test]
fn test_symbol_names_and_walks_execution() {
    let output = run_example("symbols_and_ranges/names_and_walks.rb");
    assert_eq!(output, NAMES_AND_WALKS_OUTPUT);
}

#[test]
fn test_symbol_names_and_walks_no_parens_execution() {
    let output = run_example("symbols_and_ranges/names_and_walks_no_parens.rb");
    assert_eq!(output, NAMES_AND_WALKS_OUTPUT);
}
