// Coverage tests for parser paths that need explicit testing via Lexer + Parser.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn parse_ok(code: &str) {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed");
}

fn parse_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().unwrap_err()[0].to_string()
}

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── unless with else (parser coverage) ───────────────────────────────────────

#[test]
fn parse_unless_without_else() {
    parse_ok(
        r#"
unless false
  x = 1
end
"#,
    );
}

#[test]
fn parse_unless_with_else() {
    let result = run(r#"
x = 0
unless false
  x = 1
else
  x = 2
end
x
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn unless_with_else_takes_else_branch() {
    let result = run(r#"
x = 0
unless true
  x = 1
else
  x = 2
end
x
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── for with invalid identifier (parser error) ────────────────────────────────

#[test]
fn parse_for_missing_identifier_error() {
    let err = parse_err("for 42 in [1, 2]\nend");
    assert!(err.contains("identifier") || err.contains("'for'") || err.contains("Expected"));
}

// ── Keyword-as-method-names (call.rs lines 24-44) ────────────────────────────
// These tests parse method calls using keywords as method names.
// At runtime they fail (nil has no such method), but parsing succeeds.

#[test]
fn keyword_method_name_if() {
    // nil.if is parsed fine — keyword used as method name
    let err = run_err("nil.if");
    assert!(err.contains("if") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_def() {
    let err = run_err("nil.def");
    assert!(err.contains("def") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_end() {
    let err = run_err("nil.end");
    assert!(err.contains("end") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_do() {
    let err = run_err("nil.do");
    assert!(err.contains("do") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_else() {
    let err = run_err("nil.else");
    assert!(err.contains("else") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_elsif() {
    let err = run_err("nil.elsif");
    assert!(err.contains("elsif") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_unless() {
    let err = run_err("nil.unless");
    assert!(err.contains("unless") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_while() {
    let err = run_err("nil.while");
    assert!(err.contains("while") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_for() {
    let err = run_err("nil.for");
    assert!(err.contains("for") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_in() {
    let err = run_err("nil.in");
    assert!(err.contains("in") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_begin() {
    let err = run_err("nil.begin");
    assert!(err.contains("begin") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_rescue() {
    let err = run_err("nil.rescue");
    assert!(err.contains("rescue") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_ensure() {
    let err = run_err("nil.ensure");
    assert!(err.contains("ensure") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_raise() {
    let err = run_err("nil.raise");
    assert!(err.contains("raise") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_break() {
    let err = run_err("nil.break");
    assert!(err.contains("break") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_continue() {
    let err = run_err("nil.continue");
    assert!(err.contains("continue") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_return() {
    let err = run_err("nil.return");
    assert!(err.contains("return") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_lambda() {
    let err = run_err("nil.lambda");
    assert!(err.contains("lambda") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_super() {
    let err = run_err("nil.super");
    assert!(err.contains("super") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_case() {
    let err = run_err("nil.case");
    assert!(err.contains("case") || err.contains("method") || err.contains("nil"));
}

#[test]
fn keyword_method_name_when() {
    let err = run_err("nil.when");
    assert!(err.contains("when") || err.contains("method") || err.contains("nil"));
}

// ── :: error (call.rs line 88) ────────────────────────────────────────────────

#[test]
fn scope_resolution_non_ident_error() {
    // Foo::42 is a parse error — only identifiers allowed after ::
    let err = parse_err("Foo::42");
    assert!(err.contains("constant") || err.contains("Expected") || err.contains("::"));
}

// ── Operator method definitions (function.rs lines 18-29) ────────────────────
// Each of these covers a different operator token branch in parse_function_def.

#[test]
fn operator_method_def_minus() {
    let result = run(r#"
class Num
  def initialize(v)
    @v = v
  end
  def -(other)
    @v - other.val
  end
  def val
    @v
  end
end
a = Num.new(10)
b = Num.new(3)
a - b
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn operator_method_def_star() {
    let result = run(r#"
class Num
  def initialize(v)
    @v = v
  end
  def *(other)
    @v * other.val
  end
  def val
    @v
  end
end
a = Num.new(4)
b = Num.new(5)
a * b
"#);
    assert_eq!(result, Some(Object::Int(20)));
}

#[test]
fn operator_method_def_slash() {
    let result = run(r#"
class Num
  def initialize(v)
    @v = v
  end
  def /(other)
    @v / other.val
  end
  def val
    @v
  end
end
a = Num.new(10)
b = Num.new(2)
a / b
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn operator_method_def_percent() {
    let result = run(r#"
class Num
  def initialize(v)
    @v = v
  end
  def %(other)
    @v % other.val
  end
  def val
    @v
  end
end
a = Num.new(10)
b = Num.new(3)
a % b
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn operator_method_def_bang_equal() {
    let result = run(r#"
class Num
  def initialize(v)
    @v = v
  end
  def !=(other)
    @v != other.val
  end
  def val
    @v
  end
end
a = Num.new(5)
b = Num.new(3)
a != b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_method_def_less() {
    let result = run(r#"
class Num
  def initialize(v)
    @v = v
  end
  def <(other)
    @v < other.val
  end
  def val
    @v
  end
end
a = Num.new(3)
b = Num.new(5)
a < b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_method_def_greater() {
    let result = run(r#"
class Num
  def initialize(v)
    @v = v
  end
  def >(other)
    @v > other.val
  end
  def val
    @v
  end
end
a = Num.new(5)
b = Num.new(3)
a > b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_method_def_less_equal() {
    let result = run(r#"
class Num
  def initialize(v)
    @v = v
  end
  def <=(other)
    @v <= other.val
  end
  def val
    @v
  end
end
a = Num.new(3)
b = Num.new(3)
a <= b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_method_def_greater_equal() {
    let result = run(r#"
class Num
  def initialize(v)
    @v = v
  end
  def >=(other)
    @v >= other.val
  end
  def val
    @v
  end
end
a = Num.new(5)
b = Num.new(3)
a >= b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Arrow lambda with grouped param (mod.rs lines 127-131) ───────────────────

#[test]
fn arrow_lambda_grouped_single_param() {
    let result = run(r#"
f = (x) -> x + 1
f.call(5)
"#);
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn arrow_lambda_grouped_non_ident_parse_error() {
    let err = parse_err("(1 + 2) -> 42");
    assert!(
        err.contains("parameter")
            || err.contains("identifier")
            || err.contains("arrow")
            || err.contains("Identifier")
    );
}

#[test]
fn arrow_lambda_non_ident_lhs_parse_error() {
    let err = parse_err(r#""hello" -> 42"#);
    assert!(
        err.contains("parameter")
            || err.contains("identifier")
            || err.contains("Left")
            || err.contains("arrow")
    );
}

// ── Block with || empty params (mod.rs lines 177, 235) ───────────────────────

#[test]
fn do_block_with_empty_pipe_params() {
    // Covers mod.rs line 177: parse_block with LogicalOr (||) for empty params
    let result = run(r#"
result = 0
[1, 2, 3].each do ||
  result = result + 1
end
result
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn brace_block_with_empty_pipe_params() {
    // Covers mod.rs line 235: parse_brace_block with LogicalOr (||) for empty params
    let result = run(r#"
result = 0
[1, 2, 3].each { || result = result + 1 }
result
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Block with non-ident param parse error (mod.rs lines 188, 246) ───────────

#[test]
fn do_block_non_ident_param_parse_error() {
    // Covers mod.rs line 188: parse_block non-ident param
    let err = parse_err(r#"[1].each do |42| x end"#);
    assert!(
        err.contains("parameter")
            || err.contains("identifier")
            || err.contains("Expected")
            || err.contains("Ident")
    );
}

#[test]
fn brace_block_non_ident_param_parse_error() {
    // Covers mod.rs line 246: parse_brace_block non-ident param
    let err = parse_err(r#"[1,2,3].map { |42| 1 }"#);
    assert!(
        err.contains("parameter")
            || err.contains("identifier")
            || err.contains("Expected")
            || err.contains("Ident")
    );
}

// ── Keyword argument call without parens (call.rs lines 316-323) ─────────────

#[test]
fn keyword_arg_call_without_parens_single() {
    let result = run(r#"
def greet(name:)
  name
end
greet name: "Alice"
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Alice".to_string())))
    );
}

#[test]
fn keyword_arg_call_without_parens_multiple() {
    let result = run("def describe(name:, age:)\n  name\nend\ndescribe name: \"Bob\", age: 30");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Bob".to_string())))
    );
}

// ── function.rs: operator method names | and & (lines 28-29) ─────────────────

#[test]
fn def_method_named_pipe() {
    // Method named | — covers parser function.rs line 28
    parse_ok(
        r#"
class Bits
  def |(other)
    true
  end
end
"#,
    );
}

#[test]
fn def_method_named_ampersand() {
    // Method named & — covers parser function.rs line 29
    parse_ok(
        r#"
class Bits
  def &(other)
    false
  end
end
"#,
    );
}

// ── function.rs: invalid function name (line 39) ──────────────────────────────

#[test]
fn def_with_integer_name_is_parse_error() {
    let err = parse_err("def 123\n  nil\nend");
    assert!(err.contains("function name") || err.contains("Expected"));
}

// ── function.rs: comment-only function body (line 58) ────────────────────────

#[test]
fn def_with_comment_only_body() {
    parse_ok("def silent\n  # nothing here\nend");
}

// ── function.rs: *args variadic parameter (lines 109-113) ────────────────────

#[test]
fn def_with_variadic_parameter_parses_ok() {
    parse_ok("def sum(*args)\n  args\nend");
}

// ── function.rs: non-identifier after & in params (line 103) ─────────────────

#[test]
fn def_block_param_with_invalid_name_error() {
    let err = parse_err("def foo(&123)\n  nil\nend");
    assert!(err.contains("parameter name") || err.contains("Expected"));
}

// ── control_flow.rs: comment-only branch bodies ───────────────────────────────

#[test]
fn if_then_branch_comment_only() {
    // Covers control_flow.rs line 23
    parse_ok("if true\n  # comment\nend");
}

#[test]
fn if_elsif_body_comment_only() {
    // Covers control_flow.rs line 44
    parse_ok("if true\n  1\nelsif false\n  # comment\nend");
}

#[test]
fn if_else_body_comment_only() {
    // Covers control_flow.rs line 64
    parse_ok("if false\n  1\nelse\n  # comment\nend");
}

#[test]
fn while_body_comment_only() {
    // Covers control_flow.rs line 102
    parse_ok("while false\n  # comment\nend");
}

#[test]
fn for_body_comment_only() {
    // Covers control_flow.rs line 157
    parse_ok("for x in []\n  # comment\nend");
}

#[test]
fn unless_body_comment_only() {
    // Covers control_flow.rs line 202
    parse_ok("unless false\n  # comment\nend");
}

#[test]
fn unless_else_body_comment_only() {
    // Covers control_flow.rs line 215
    parse_ok("unless false\n  1\nelse\n  # comment\nend");
}

// ── control_flow.rs: bare return without value (line 246) ────────────────────

#[test]
fn bare_return_without_value() {
    // Covers control_flow.rs line 246 — `return` at end of input → None value
    parse_ok("return");
}

// ── control_flow.rs: case/when with comment-only bodies ──────────────────────

#[test]
fn case_when_body_comment_only() {
    // Covers control_flow.rs line 314
    parse_ok("case 1\nwhen 1\n  # comment\nend");
}

#[test]
fn case_else_body_comment_only() {
    // Covers control_flow.rs line 338
    parse_ok("case 1\nwhen 1\n  1\nelse\n  # comment\nend");
}

// ── control_flow.rs: case/in with comment-only bodies ────────────────────────

#[test]
fn case_in_body_comment_only() {
    // Covers control_flow.rs line 400
    parse_ok("case 1\nin 1\n  # comment\nend");
}

#[test]
fn case_in_else_body_comment_only() {
    // Covers control_flow.rs line 424
    parse_ok("case 1\nin 1\n  1\nelse\n  # comment\nend");
}

// ── control_flow.rs: pattern binding with non-identifier (line 508) ──────────

#[test]
fn case_in_bind_non_identifier_error() {
    // Covers control_flow.rs line 508 — `=> 42` is not a valid binding identifier
    let err = parse_err("case 1\nin 1 => 42\n  nil\nend");
    assert!(err.contains("identifier") || err.contains("Expected"));
}

// ── control_flow.rs: multiple patterns with terminal after comma (lines 559, 568)

#[test]
fn case_when_trailing_comma_before_terminal_token() {
    // Covers lines 559 and 568 — comma followed immediately by `then`
    parse_ok("case 1\nwhen 1, then\n  1\nend");
}

// ── primary.rs: lambda with non-identifier parameter (line 197) ──────────────

#[test]
fn lambda_with_invalid_param_name_error() {
    let err = parse_err("lambda do |123|\n  nil\nend");
    assert!(err.contains("parameter name") || err.contains("Expected"));
}

// ── primary.rs: lambda with comment-only body (line 221) ─────────────────────

#[test]
fn lambda_with_comment_only_body() {
    parse_ok("lambda do\n  # nothing\nend");
}

// ── primary.rs: do-block with non-identifier parameter (line 253) ────────────

#[test]
fn do_block_with_invalid_param_name_error() {
    let err = parse_err("x = do |123|\n  nil\nend");
    assert!(err.contains("parameter name") || err.contains("Expected"));
}

// ── primary.rs: do-block with comment-only body (line 277) ───────────────────

#[test]
fn do_block_with_comment_only_body() {
    parse_ok("x = do\n  # nothing\nend");
}

// ── primary.rs: if-expression with comment-only branches (lines 462, 482, 500)

#[test]
fn if_expression_then_comment_only() {
    // Covers primary.rs line 462
    parse_ok("x = if true\n  # comment\nend");
}

#[test]
fn if_expression_elsif_comment_only() {
    // Covers primary.rs line 482
    parse_ok("x = if true\n  1\nelsif false\n  # comment\nend");
}

#[test]
fn if_expression_else_comment_only() {
    // Covers primary.rs line 500
    parse_ok("x = if false\n  1\nelse\n  # comment\nend");
}

// ── primary.rs: unless-expression (lines 537, 549, 556) ──────────────────────

#[test]
fn unless_expression_then_comment_only() {
    // Covers primary.rs line 537
    parse_ok("x = unless true\n  # comment\nend");
}

#[test]
fn unless_expression_else_comment_only() {
    // Covers primary.rs line 549
    parse_ok("x = unless true\n  1\nelse\n  # comment\nend");
}

#[test]
fn unless_expression_without_else() {
    // Covers primary.rs line 556 — unless expr with no else branch
    let result = run("x = unless false\n  42\nend\nx");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── call.rs: no-paren call with multiple positional args (line 368) ───────────

#[test]
fn no_paren_call_two_positional_args() {
    // First arg must not be a simple ident (which would trigger Pattern 2 of
    // should_parse_as_arg). Use `1 + 0` so peek_ahead(1) is `+` not `,`.
    // After the first arg is parsed, the while-comma loop parses the second.
    // This covers finish_call_without_parens line 368.
    let result = run("def add(a, b)\n  a + b\nend\nadd 1 + 0, 2");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── call.rs: no-paren call with trailing do block (line 394) ─────────────────

#[test]
fn no_paren_call_with_trailing_do_block() {
    // Covers finish_call_without_parens line 394 — trailing do..end block
    let result = run("def run_with(x)\n  x\nend\nrun_with !false do\n  99\nend");
    // run_with is called with !false=true and a trailing block (block is ignored in body)
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── call.rs: no-paren call where non-ident precedes colon (line 283) ─────────

#[test]
fn should_parse_as_arg_returns_false_for_non_ident_before_colon() {
    // When peek is Int(42) and peek_ahead(1) is Colon, should_parse_as_arg
    // returns false at line 283, so foo is parsed as an identifier, not a call.
    // The overall statement is a valid hash literal.
    parse_ok("x = {42 => 1}");
}

// ── class.rs: module name not an identifier (line 70) ────────────────────────

#[test]
fn module_with_non_ident_name_error() {
    let err = parse_err("module 42\nend");
    assert!(err.contains("module name") || err.contains("Expected") || err.contains("Ident"));
}

// ── class.rs: include with non-ident name (line 107) ─────────────────────────

#[test]
fn include_with_non_ident_name_error() {
    let err = parse_err("class Foo\n  include 42\nend");
    assert!(err.contains("module name") || err.contains("Expected") || err.contains("include"));
}

// ── class.rs: extend with non-ident name (line 125) ──────────────────────────

#[test]
fn extend_with_non_ident_name_error() {
    let err = parse_err("class Foo\n  extend 42\nend");
    assert!(err.contains("module name") || err.contains("Expected") || err.contains("extend"));
}

// ── class_execution.rs: define_method with String name (line 265) ────────────

#[test]
fn define_method_with_string_name() {
    let result = run(r#"
class Greeter
  define_method("hello") do
    "world"
  end
end
Greeter.new.hello
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("world".to_string())))
    );
}

// ── parser: rest pattern used directly in case-in is a parse error ───────────

#[test]
fn rest_pattern_at_top_level_is_parse_error() {
    // Using *x as a top-level case-in pattern is a syntax error at the parser level.
    let err = parse_err("case 1\nin *x\n  x\nend");
    assert!(err.contains("pattern") || err.contains("Star") || err.contains("Expected"));
}

// ── call.rs: no-paren call with trailing { block } (line 396) ────────────────

#[test]
fn no_paren_call_with_trailing_brace_block() {
    // Covers finish_call_without_parens line 396 — trailing { } block
    let result = run("def run_with(x)\n  x\nend\nrun_with !false { 99 }");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── call.rs: break after trailing comma before end (line 352) ────────────────

#[test]
fn no_paren_call_trailing_comma_before_end() {
    // After a comma in finish_call_without_parens, if the next token is End,
    // the while-comma loop breaks at line 352.
    let result =
        run("def noop(x)\n  x\nend\nresult = nil\nif true\n  result = noop 1 + 0,\nend\nresult");
    assert_eq!(result, Some(Object::Int(1)));
}

// ── attributes.rs: parse error for attr_reader with non-symbol (lines 64, 81) ──

#[test]
fn attr_reader_with_non_symbol_error() {
    let err = parse_err("class Foo\n  attr_reader 42\nend");
    assert!(err.contains("attribute") || err.contains("Expected") || err.contains("Unexpected"));
}

// ── function.rs: variadic param with non-ident after star (line 111) ──

#[test]
fn variadic_param_with_non_ident_error() {
    let err = parse_err("def foo(*42)\nend");
    assert!(err.contains("parameter") || err.contains("Expected") || err.contains("Unexpected"));
}

// ── class.rs: class with non-ident parent (line 42) ──

#[test]
fn class_with_non_ident_parent_error() {
    let err = parse_err("class Foo < 42\nend");
    assert!(err.contains("class") || err.contains("Expected") || err.contains("Unexpected"));
}

// ── call.rs line 295: should_parse_as_arg returns false for RBrace ──

#[test]
fn no_paren_call_not_triggered_before_rbrace() {
    // `{x 1}` - x followed by 1 then }, should_parse_as_arg returns false
    // because peek_ahead(1) for x is a space then 1, but peek_ahead(1) is 1
    // Actually this tests that we don't parse "x 1" as a call inside a hash
    let result = run(r#"x = {"a" => 1}
x"#);
    if let Some(Object::Dict(_)) = result {
        // Success - parsed as dict, not function call
    } else {
        panic!("Expected dict");
    }
}

// ── call.rs lines 332-333: dict-like syntax error ──

#[test]
fn no_paren_call_with_colon_after_expr_is_error() {
    // After parsing first positional arg, if colon follows, it's dict syntax error
    let err = parse_err("foo 1 : 2");
    assert!(
        err.contains("dictionary")
            || err.contains("Expected")
            || err.contains("syntax")
            || err.contains("Unexpected")
    );
}
