// Examples covering the forms the parser reads

use super::run_example;

/// The expected output of both `syntax/reading_forms` variants, which differ
/// only in whether the calls are written with parentheses.
const READING_FORMS_OUTPUT: &str = "\"matched\"\n\"kept\"\n\"nil\"\n[0, nil, 2]\n{nil => nil}\n3\n[\"a\", \"3\", \"c\"]\n[:a, :b3]\n[\"x\", \"y\"]\n[:x, :y]\n\"a\\#{1}b\"\n\"a(b\"\n[4, 6]\nnil\n5\nnil\n[2, 3]\n";

#[test]
fn test_syntax_reading_forms_execution() {
    let output = run_example("syntax/reading_forms.rb");
    assert_eq!(output, READING_FORMS_OUTPUT);
}

#[test]
fn test_syntax_reading_forms_no_parens_execution() {
    let output = run_example("syntax/reading_forms_no_parens.rb");
    assert_eq!(output, READING_FORMS_OUTPUT);
}

/// The expected output of both `syntax/splat_assignment` variants.
const SPLAT_ASSIGNMENT_OUTPUT: &str = "1\n[2, 3]\n1\n[]\n[1, 2]\n3\n1\n[2, 3, 4]\n5\n1\n[]\n2\n[1, 2]\n9\n[]\n1\n[2, 3]\n\"first\"\n[\"second\", \"third\"]\n";

#[test]
fn test_syntax_splat_assignment_execution() {
    let output = run_example("syntax/splat_assignment.rb");
    assert_eq!(output, SPLAT_ASSIGNMENT_OUTPUT);
}

#[test]
fn test_syntax_splat_assignment_no_parens_execution() {
    let output = run_example("syntax/splat_assignment_no_parens.rb");
    assert_eq!(output, SPLAT_ASSIGNMENT_OUTPUT);
}

/// The expected output of both `syntax/escape_runs/bytes` variants.
const ESCAPE_RUNS_OUTPUT: &str =
    "\"ロ\"\n\"ロ\"\n[227, 131, 173]\n[0]\n\"aAb\"\n5\n\"tab\\there\"\n\"ロ\"\n[27, 91, 49, 109]\n";

#[test]
fn test_syntax_escape_runs_bytes_execution() {
    let output = run_example("syntax/escape_runs/bytes.rb");
    assert_eq!(output, ESCAPE_RUNS_OUTPUT);
}

#[test]
fn test_syntax_escape_runs_bytes_parens_execution() {
    let output = run_example("syntax/escape_runs/bytes_parens.rb");
    assert_eq!(output, ESCAPE_RUNS_OUTPUT);
}

/// The expected output of both `syntax/binary_source/bytes` variants.
const BINARY_SOURCE_OUTPUT: &str = "3\n3\n3\n";

#[test]
fn test_syntax_binary_source_bytes_execution() {
    let output = run_example("syntax/binary_source/bytes.rb");
    assert_eq!(output, BINARY_SOURCE_OUTPUT);
}

#[test]
fn test_syntax_binary_source_bytes_parens_execution() {
    let output = run_example("syntax/binary_source/bytes_parens.rb");
    assert_eq!(output, BINARY_SOURCE_OUTPUT);
}

/// The expected output of both `syntax/grouped_assignment` variants, which
/// differ only in whether the calls are written with parentheses.
const GROUPED_ASSIGNMENT_OUTPUT: &str = "[1, 2]\n[1, 2]\n1\n[1, nil, nil]\n\"assignment\"\n\"assignment\"\n\"assignment\"\n\"assignment\"\n\"method\"\n[]\n";

#[test]
fn test_syntax_grouped_assignment_execution() {
    let output = run_example("syntax/grouped_assignment.rb");
    assert_eq!(output, GROUPED_ASSIGNMENT_OUTPUT);
}

#[test]
fn test_syntax_grouped_assignment_no_parens_execution() {
    let output = run_example("syntax/grouped_assignment_no_parens.rb");
    assert_eq!(output, GROUPED_ASSIGNMENT_OUTPUT);
}

/// The expected output of both `syntax/retry_and_rescue` variants.
const RETRY_AND_RESCUE_OUTPUT: &str = "[4, :done]\n[2, 4]\nrefused outside a rescue\n";

#[test]
fn test_syntax_retry_and_rescue_execution() {
    let output = run_example("syntax/retry_and_rescue.rb");
    assert_eq!(output, RETRY_AND_RESCUE_OUTPUT);
}

#[test]
fn test_syntax_retry_and_rescue_parens_execution() {
    let output = run_example("syntax/retry_and_rescue_parens.rb");
    assert_eq!(output, RETRY_AND_RESCUE_OUTPUT);
}

/// The expected output of both `syntax/grouped_targets` variants, which differ only in whether
/// the calls are written with parentheses.
const GROUPED_TARGETS_OUTPUT: &str = "[1, 2, 3]\n[1, nil, nil, nil, nil, nil, nil, nil]\n[4, [5, 6]]\n[nil]\n[7, 8, 9]\n[3, 4, 1, 2]\n[:a, :b]\n[:one, :two]\nnil\n2\n";

#[test]
fn test_syntax_grouped_targets_execution() {
    let output = run_example("syntax/grouped_targets.rb");
    assert_eq!(output, GROUPED_TARGETS_OUTPUT);
}

#[test]
fn test_syntax_grouped_targets_parens_execution() {
    let output = run_example("syntax/grouped_targets_parens.rb");
    assert_eq!(output, GROUPED_TARGETS_OUTPUT);
}

/// The expected output of both `syntax/numbered_and_shorthand` variants, which show
/// a block that names its arguments `_1`, and a call that passes a keyword by name alone and differ only in whether the calls are
/// written with parentheses.
const NUMBERED_AND_SHORTHAND_OUTPUT: &str =
    "[2, 4, 6]\n[[1, 10], [2, 20]]\n[4, 6, 10]\n[1, 6, 7]\n{first: 4, second: 6}\n";

#[test]
fn test_syntax_numbered_and_shorthand_execution() {
    let output = run_example("syntax/numbered_and_shorthand.rb");
    assert_eq!(output, NUMBERED_AND_SHORTHAND_OUTPUT);
}

#[test]
fn test_syntax_numbered_and_shorthand_no_parens_execution() {
    let output = run_example("syntax/numbered_and_shorthand_no_parens.rb");
    assert_eq!(output, NUMBERED_AND_SHORTHAND_OUTPUT);
}

/// The expected output of both `syntax/do_block_binding` variants, which
/// differ only in whether the calls are written with parentheses.
const DO_BLOCK_BINDING_OUTPUT: &str =
    concat!("\"outer got the block\"\n", "\"outer got the block\"\n",);

#[test]
fn test_syntax_do_block_binding_execution() {
    let output = run_example("syntax/do_block_binding.rb");
    assert_eq!(output, DO_BLOCK_BINDING_OUTPUT);
}

#[test]
fn test_syntax_do_block_binding_no_parens_execution() {
    let output = run_example("syntax/do_block_binding_no_parens.rb");
    assert_eq!(output, DO_BLOCK_BINDING_OUTPUT);
}

/// The expected output of both `syntax/operator_binding` variants, which
/// differ only in whether the calls are written with parentheses.
const OPERATOR_BINDING_OUTPUT: &str = concat!(
    "1..10\n1...10\n",
    "a range takes no range\n",
    "a relation takes no relation\n",
    "a comparison takes no comparison\n",
    "10\n[4]\n"
);

#[test]
fn test_syntax_operator_binding_execution() {
    let output = run_example("syntax/operator_binding.rb");
    assert_eq!(output, OPERATOR_BINDING_OUTPUT);
}

#[test]
fn test_syntax_operator_binding_parens_execution() {
    let output = run_example("syntax/operator_binding_parens.rb");
    assert_eq!(output, OPERATOR_BINDING_OUTPUT);
}

/// The expected output of both `syntax/keyword_symbols_in_when` variants.
const KEYWORD_SYMBOLS_IN_WHEN_OUTPUT: &str =
    concat!("method\n", "keyword\n", "other\n", "pattern\n");

#[test]
fn test_syntax_keyword_symbols_in_when_execution() {
    let output = run_example("syntax/keyword_symbols_in_when.rb");
    assert_eq!(output, KEYWORD_SYMBOLS_IN_WHEN_OUTPUT);
}

#[test]
fn test_syntax_keyword_symbols_in_when_no_parens_execution() {
    let output = run_example("syntax/keyword_symbols_in_when_no_parens.rb");
    assert_eq!(output, KEYWORD_SYMBOLS_IN_WHEN_OUTPUT);
}

/// The expected output of both `syntax/assignment_order` variants.
const ASSIGNMENT_ORDER_OUTPUT: &str = "[:receiver, :value]\n[:receiver, :key, :value]\n[:namespace, :value]\n80\n[:a, :b, :c, :d]\n[:a, :b]\n5\n[:receiver]\n3\n[:receiver, :key]\n15\n[:namespace]\n2\n7\n9\n[1, 2]\n4\nnil\nnil\nsingle x 1\n\":not_a_module is not a class/module\"\n1\nNoMethodError\ntrue\ntrue\n2\nnil\nSyntaxError\n";

#[test]
fn test_syntax_assignment_order_execution() {
    let output = run_example("syntax/assignment_order.rb");
    assert_eq!(output, ASSIGNMENT_ORDER_OUTPUT);
}

#[test]
fn test_syntax_assignment_order_no_parens_execution() {
    let output = run_example("syntax/assignment_order_no_parens.rb");
    assert_eq!(output, ASSIGNMENT_ORDER_OUTPUT);
}

/// The expected output of both `syntax/block_parameters` variants.
const BLOCK_PARAMETERS_OUTPUT: &str = "[1, [], 2, nil]\n[5, 1, 2, nil]\n[1, 2, 6, 3, 4]\n[1, 2, 3, 4, 5]\n[1, 2, 6, [], 3, 4]\n[:left, :right, nil]\n[:left, nil, :right]\n[true, nil]\nasked to_ary true\n3\n12\n\"can't convert Wrong to Array (Wrong#to_ary gives Integer)\"\n1\nnil\n:outer\n[1, nil]\nrefused: [1].each { |x, x| }\nrefused: -> (x, x) {}\nrefused: [1].each { |a; a| }\nrefused: [1].each { |a; b; c| }\nrefused: def lone; hand(1, &); end\n\"10 and 20\"\n";

#[test]
fn test_syntax_block_parameters_execution() {
    let output = run_example("syntax/block_parameters.rb");
    assert_eq!(output, BLOCK_PARAMETERS_OUTPUT);
}

#[test]
fn test_syntax_block_parameters_no_parens_execution() {
    let output = run_example("syntax/block_parameters_no_parens.rb");
    assert_eq!(output, BLOCK_PARAMETERS_OUTPUT);
}

/// The expected output of both `syntax/constant_names` variants.
const CONSTANT_NAMES_OUTPUT: &str = "1\n1\n2\nModule\n3\n";

#[test]
fn test_syntax_constant_names_execution() {
    let output = run_example("syntax/constant_names.rb");
    assert_eq!(output, CONSTANT_NAMES_OUTPUT);
}

#[test]
fn test_syntax_constant_names_no_parens_execution() {
    let output = run_example("syntax/constant_names_no_parens.rb");
    assert_eq!(output, CONSTANT_NAMES_OUTPUT);
}

/// The expected output of both `syntax/special_globals` variants.
const SPECIAL_GLOBALS_OUTPUT: &str = "[TypeError, \"wrong argument type Object (expected MatchData)\"]\n[TypeError, \"$stdout must have write method, NilClass given\"]\n[NameError, \"$! is a read-only variable\"]\n[NameError, \"$FILENAME is a read-only variable\"]\n[TypeError, \"value of $/ must be String\"]\n[TypeError, \"value of $-0 must be String\"]\n[TypeError, \"no implicit conversion of nil into String\"]\n[TypeError, \"no implicit conversion from nil to integer\"]\n[ArgumentError, \"$! not set\"]\n[String, true, true]\ntrue\n12\n7\ntrue\n[true, true, true, true]\n[\"here:1\"]\n\"Can't assign to nil\"\n\"Can't assign to true\"\n\"Can't assign to false\"\n\"Can't change the value of self\"\n\"Can't set variable $&\"\n\"Can't set variable $1\"\n[NameError, \"$matched_text is a read-only variable\"]\n[#<Encoding:ISO-8859-1>, #<Encoding:ISO-8859-1>, #<Encoding:ISO-8859-1>, #<Encoding:ISO-8859-1>]\n[\"inner\", \"inner\"]\n\"main line\"\n[:rb, \"pp.rb\"]\nnil\n[true, false]\n[\"variable $= is no longer effective\\n\", \"variable $= is no longer effective; ignored\\n\", \"non-nil '$,' is deprecated\\n\"]\n";

#[test]
fn test_syntax_special_globals_execution() {
    let output = run_example("syntax/special_globals.rb");
    assert_eq!(output, SPECIAL_GLOBALS_OUTPUT);
}

#[test]
fn test_syntax_special_globals_no_parens_execution() {
    let output = run_example("syntax/special_globals_no_parens.rb");
    assert_eq!(output, SPECIAL_GLOBALS_OUTPUT);
}

/// The expected output of both `syntax/multiple_assignment_conversions` variants.
const MULTIPLE_ASSIGNMENT_CONVERSIONS_OUTPUT: &str = "[1, 2, nil]\n[1, [2]]\n[3, 4]\n[true, nil]\n[true, nil]\n\"can't convert Object to Array (Object#to_ary gives Integer)\"\n[5, 6]\n[1, 7]\n[[7, 8], false]\nfalse\n\"can't convert Object to Array (Object#to_a gives Integer)\"\n[1, 1, 2, 4]\n[\"local-variable\", nil]\n[]\n[\"global variable '$never_assigned_global' not initialized\\n\"]\n\"-\"\n";

#[test]
fn test_syntax_multiple_assignment_conversions_execution() {
    let output = run_example("syntax/multiple_assignment_conversions.rb");
    assert_eq!(output, MULTIPLE_ASSIGNMENT_CONVERSIONS_OUTPUT);
}

#[test]
fn test_syntax_multiple_assignment_conversions_no_parens_execution() {
    let output = run_example("syntax/multiple_assignment_conversions_no_parens.rb");
    assert_eq!(output, MULTIPLE_ASSIGNMENT_CONVERSIONS_OUTPUT);
}

/// The expected output of both `syntax/interpolated_statements` variants.
const INTERPOLATED_STATEMENTS_OUTPUT: &str =
    "\"keyword\"\n\"20\"\n\"total: 5\"\n\"inner 2\"\n\"rescued\"\n:symbol1\n";

#[test]
fn test_syntax_interpolated_statements_execution() {
    let output = run_example("syntax/interpolated_statements.rb");
    assert_eq!(output, INTERPOLATED_STATEMENTS_OUTPUT);
}

#[test]
fn test_syntax_interpolated_statements_no_parens_execution() {
    let output = run_example("syntax/interpolated_statements_no_parens.rb");
    assert_eq!(output, INTERPOLATED_STATEMENTS_OUTPUT);
}

/// The expected output of both `syntax/statement_conditionals` variants.
const STATEMENT_CONDITIONALS_OUTPUT: &str =
    "[\"zero\", \"small\", \"large\"]\nLibrary::NameError\n";

#[test]
fn test_syntax_statement_conditionals_execution() {
    let output = run_example("syntax/statement_conditionals.rb");
    assert_eq!(output, STATEMENT_CONDITIONALS_OUTPUT);
}

#[test]
fn test_syntax_statement_conditionals_no_parens_execution() {
    let output = run_example("syntax/statement_conditionals_no_parens.rb");
    assert_eq!(output, STATEMENT_CONDITIONALS_OUTPUT);
}

/// The expected output of both `syntax/keyword_labels` variants.
const KEYWORD_LABELS_OUTPUT: &str = concat!(
    "{if: :ready, next: 2, class: \"primary\", end: 10, self: true, not: false}\n",
    "12\n",
    "[true, 5, \"UTC\"]\n",
    "[true, 9, \"CET\"]\n",
    "[:not, :and, :or, :in, :__FILE__, :__LINE__, :__dir__, :__ENCODING__, :defined?]\n",
    "0\n",
    "20\n",
    "12\n",
);

#[test]
fn test_syntax_keyword_labels_execution() {
    let output = run_example("syntax/keyword_labels.rb");
    assert_eq!(output, KEYWORD_LABELS_OUTPUT);
}

#[test]
fn test_syntax_keyword_labels_no_parens_execution() {
    let output = run_example("syntax/keyword_labels_no_parens.rb");
    assert_eq!(output, KEYWORD_LABELS_OUTPUT);
}

/// The expected output of both `syntax/locals_by_scope` variants.
const LOCALS_BY_SCOPE_OUTPUT: &str = concat!(
    "20\n",
    "[:called, [[1]]]\n",
    "[:class_bar, [[2]]]\n",
    "[:method_baz, [[3]]]\n",
    "\"e\"\n",
    "0\n",
    "[:grault, [[4]]]\n",
    "7\n",
);

#[test]
fn test_syntax_locals_by_scope_execution() {
    let output = run_example("syntax/locals_by_scope.rb");
    assert_eq!(output, LOCALS_BY_SCOPE_OUTPUT);
}

#[test]
fn test_syntax_locals_by_scope_no_parens_execution() {
    let output = run_example("syntax/locals_by_scope_no_parens.rb");
    assert_eq!(output, LOCALS_BY_SCOPE_OUTPUT);
}

/// The expected output of both `syntax/lambda_parameter` variants.
const LAMBDA_PARAMETER_OUTPUT: &str = "[:lambda, 1]\n[:proc, 2]\n";

#[test]
fn test_syntax_lambda_parameter_execution() {
    let output = run_example("syntax/lambda_parameter.rb");
    assert_eq!(output, LAMBDA_PARAMETER_OUTPUT);
}

#[test]
fn test_syntax_lambda_parameter_no_parens_execution() {
    let output = run_example("syntax/lambda_parameter_no_parens.rb");
    assert_eq!(output, LAMBDA_PARAMETER_OUTPUT);
}

/// The expected output of both `syntax/safe_navigation_operators` variants.
const SAFE_NAVIGATION_OPERATORS_OUTPUT: &str = concat!(
    "3\n",
    "4\n",
    "{tea: 3, coffee: 4}\n",
    "10\n",
    "nil\n",
    "nil\n",
    "nil\n",
    ":none\n",
);

#[test]
fn test_syntax_safe_navigation_operators_execution() {
    let output = run_example("syntax/safe_navigation_operators.rb");
    assert_eq!(output, SAFE_NAVIGATION_OPERATORS_OUTPUT);
}

#[test]
fn test_syntax_safe_navigation_operators_no_parens_execution() {
    let output = run_example("syntax/safe_navigation_operators_no_parens.rb");
    assert_eq!(output, SAFE_NAVIGATION_OPERATORS_OUTPUT);
}

const ADJACENT_STRING_LITERALS: &str = concat!(
    "\"ab\"\n",
    "\"a3\"\n",
    "\"3b\"\n",
    "3\n",
    "\"a3c\"\n",
    "3\n",
    "\"x6y3\"\n",
    "\"first part, second part 3\"\n",
);

#[test]
fn test_syntax_adjacent_string_literals_execution() {
    let output = run_example("syntax/adjacent_string_literals.rb");
    assert_eq!(output, ADJACENT_STRING_LITERALS);
}

#[test]
fn test_syntax_adjacent_string_literals_no_parens_execution() {
    let output = run_example("syntax/adjacent_string_literals_no_parens.rb");
    assert_eq!(output, ADJACENT_STRING_LITERALS);
}
