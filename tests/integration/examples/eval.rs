use super::run_example;

const SCOPE_AND_RETURNS: &str = "\"Cabinet::Shelf\"\nKernel\ntrue\n6\nLocalJumpError\n:from_the_eval\nLocalJumpError\nnil\n#<Encoding:BINARY (ASCII-8BIT)>\ntrue\n12\n42\n\"wrong argument type proc (expected binding)\"\n";

#[test]
fn test_eval_scope_and_returns_execution() {
    let output = run_example("eval/eval_scope_and_returns.rb");
    assert_eq!(output, SCOPE_AND_RETURNS);
}

#[test]
fn test_eval_scope_and_returns_no_parens_execution() {
    let output = run_example("eval/eval_scope_and_returns_no_parens.rb");
    assert_eq!(output, SCOPE_AND_RETURNS);
}

/// The expected output of both `eval/binding_paths` variants.
const BINDING_PATHS_OUTPUT: &str = "true\ntrue\ntrue\ntrue\n";

#[test]
fn test_eval_binding_paths_execution() {
    let output = run_example("eval/binding_paths.rb");
    assert_eq!(output, BINDING_PATHS_OUTPUT);
}

#[test]
fn test_eval_binding_paths_no_parens_execution() {
    let output = run_example("eval/binding_paths_no_parens.rb");
    assert_eq!(output, BINDING_PATHS_OUTPUT);
}

/// The expected output of both `eval/locals_read_while_parsing` variants.
const LOCALS_READ_WHILE_PARSING_OUTPUT: &str = concat!(
    "\"[:method, [[1]]]\"\n",
    "\"20\"\n",
    ":\"10\"\n",
    "/10/\n",
    "\"1\"\n",
    "20\n",
    "10\n",
    "20\n",
    "10\n",
    "4\n",
    "\"[:method, [[1]]]\"\n",
);

#[test]
fn test_eval_locals_read_while_parsing_execution() {
    let output = run_example("eval/locals_read_while_parsing.rb");
    assert_eq!(output, LOCALS_READ_WHILE_PARSING_OUTPUT);
}

#[test]
fn test_eval_locals_read_while_parsing_no_parens_execution() {
    let output = run_example("eval/locals_read_while_parsing_no_parens.rb");
    assert_eq!(output, LOCALS_READ_WHILE_PARSING_OUTPUT);
}

/// The expected output of both `eval/syntax_error_reports` variants.
const SYNTAX_ERROR_REPORTS: &str = concat!(
    "(eval):1: syntax error found\n",
    "> 1 | proc { _1 = 0 }\n",
    "    |        ^~ _1 is reserved for numbered parameters\n",
    "(eval):1: syntax errors found\n",
    "> 1 | x = 1 +\n",
    "    |        ^ unexpected end-of-input, assuming it is closing the parent top level context\n",
    "    |        ^ unexpected end-of-input; expected an expression after the operator\n",
    "(eval):4: syntax errors found\n",
    "  2 | second\n",
    "  3 | third\n",
    "> 4 | value = = 1\n",
    "    |       ^ expected an expression after `=`\n",
    "    |         ^ unexpected '='; target cannot be written\n",
    "    |           ^ unexpected integer, expecting end-of-input\n",
    "  5 | fourth\n",
    "  6 | fifth\n",
    "(eval):1: syntax errors found\n",
    "> 1 | ... + ) + 1\n",
    "    |     ^ unexpected ')', expected a receiver for unary `+`\n",
    "    |       ^ unexpected ')', ignoring it\n",
    "    |       ^ unexpected ')', expecting end-of-input\n",
    "    |         ^ unexpected '+', ignoring it\n",
    "(eval):1: syntax errors found\n",
    "> 1 | ä = 1 +\n",
    "    |        ^ unexpected end-of-input, assuming it is closing the parent top level context\n",
    "    |        ^ unexpected end-of-input; expected an expression after the operator\n",
    "named.rb:10: syntax errors found\n",
    "> 10 | 1 +\n",
    "     |    ^ unexpected end-of-input, assuming it is closing the parent top level context\n",
    "     |    ^ unexpected end-of-input; expected an expression after the operator\n",
);

#[test]
fn test_eval_syntax_error_reports_execution() {
    let output = run_example("eval/syntax_error_reports.rb");
    assert_eq!(output, SYNTAX_ERROR_REPORTS);
}

#[test]
fn test_eval_syntax_error_reports_no_parens_execution() {
    let output = run_example("eval/syntax_error_reports_no_parens.rb");
    assert_eq!(output, SYNTAX_ERROR_REPORTS);
}

/// The expected output of both `eval/binding_in_module_eval` variants.
const BINDING_IN_MODULE_EVAL: &str =
    concat!("[:answer]\n", "[:define_on]\n", "42\n", "\"2 items\\n\"\n",);

#[test]
fn test_eval_binding_in_module_eval_execution() {
    let output = run_example("eval/binding_in_module_eval.rb");
    assert_eq!(output, BINDING_IN_MODULE_EVAL);
}

#[test]
fn test_eval_binding_in_module_eval_no_parens_execution() {
    let output = run_example("eval/binding_in_module_eval_no_parens.rb");
    assert_eq!(output, BINDING_IN_MODULE_EVAL);
}

/// The expected output of both `eval/refused_programs` variants.
const REFUSED_PROGRAMS: &str = concat!(
    "(eval):1: syntax error found\n",
    "> 1 | 1 2\n",
    "    |   ^ unexpected integer, expecting end-of-input\n",
    "(eval):2: syntax error found\n",
    "  1 | first = 1\n",
    "> 2 | second = 2 3\n",
    "    |            ^ unexpected integer, expecting end-of-input\n",
    "(eval):1: syntax error found\n",
    "> 1 | def pair(left, left); end\n",
    "    |                ^~~~ duplicated argument name\n",
    "2\n",
);

#[test]
fn test_eval_refused_programs_execution() {
    let output = run_example("eval/refused_programs.rb");
    assert_eq!(output, REFUSED_PROGRAMS);
}

#[test]
fn test_eval_refused_programs_no_parens_execution() {
    let output = run_example("eval/refused_programs_no_parens.rb");
    assert_eq!(output, REFUSED_PROGRAMS);
}
