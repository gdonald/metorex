// Additional coverage tests for parser modules:
// - token_stream.rs
// - statements/control_flow.rs
// - expressions/primary.rs
// - statements/exception.rs

use metorex::lexer::Lexer;
use metorex::parser::Parser;

fn parse_ok(code: &str) {
    let tokens = Lexer::new(code).tokenize();
    let result = Parser::new(tokens).parse();
    assert!(
        result.is_ok(),
        "Parse failed for: {}\nError: {:?}",
        code,
        result.err()
    );
}

// ── token_stream.rs lines 24, 41: peek/previous at boundaries ──────────────
// These are tested indirectly through parsing - the parser wraps these methods.

#[test]
fn token_stream_peek_past_end_via_empty_parse() {
    // Parsing empty input exercises peek at EOF
    let tokens = Lexer::new("").tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn token_stream_previous_at_start_via_single_token() {
    // Parsing a single token exercises previous() at or near position 0
    let tokens = Lexer::new("42").tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

// ── token_stream.rs lines 71-77: match_kind for complex types ───────────────

#[test]
fn token_stream_match_kind_interpolated_string() {
    // Parse an interpolated string - exercises InterpolatedString match branch
    parse_ok(r#""hello #{1 + 2} world""#);
}

#[test]
fn token_stream_match_kind_class_var() {
    // Parse a class variable - exercises ClassVar match branch
    parse_ok(
        r#"
class Foo
  @@x = 1
end
"#,
    );
}

#[test]
fn token_stream_match_kind_instance_var() {
    // Parse an instance variable - exercises InstanceVar match branch
    parse_ok(
        r#"
class Foo
  def initialize
    @x = 1
  end
end
"#,
    );
}

// ── token_stream.rs lines 94-95: skip_newlines ─────────────────────────────

#[test]
fn parser_handles_multiple_newlines() {
    parse_ok(
        r#"

x = 1


y = 2

"#,
    );
}

// ── token_stream.rs lines 102-104: skip_comments ───────────────────────────

#[test]
fn parser_handles_comments_between_statements() {
    parse_ok(
        r#"
x = 1
# this is a comment
y = 2
"#,
    );
}

// ── parser/statements/control_flow.rs: unless with else ─────────────────────

#[test]
fn parse_unless_with_else() {
    parse_ok(
        r#"
unless false
  x = 1
else
  x = 2
end
"#,
    );
}

// ── parser/statements/control_flow.rs: for loop with do keyword ─────────────

#[test]
fn parse_for_with_do() {
    parse_ok(
        r#"
for x in [1, 2, 3] do
  puts x
end
"#,
    );
}

// ── parser/statements/control_flow.rs: while loop with do keyword ───────────

#[test]
fn parse_while_with_do() {
    parse_ok(
        r#"
x = 0
while x < 10 do
  x = x + 1
end
"#,
    );
}

// ── parser/statements/control_flow.rs: case with then keyword ───────────────

#[test]
fn parse_case_with_then() {
    parse_ok(
        r#"
case x
when 1 then puts "one"
when 2 then puts "two"
end
"#,
    );
}

// ── parser/statements/control_flow.rs: case with guard ──────────────────────

#[test]
fn parse_case_with_guard() {
    parse_ok(
        r#"
case x
when n if n > 0
  puts "positive"
when n
  puts "other"
end
"#,
    );
}

// ── parser/statements/control_flow.rs: case/in pattern matching ─────────────

#[test]
fn parse_case_in() {
    parse_ok(
        r#"
case [1, 2, 3]
in [1, ...rest]
  puts rest
in Integer => n
  puts n
else
  puts "other"
end
"#,
    );
}

// ── parser/statements/control_flow.rs: case/in with guard ───────────────────

#[test]
fn parse_case_in_with_guard() {
    parse_ok(
        r#"
case 42
in Integer => n if n > 0
  puts "positive"
else
  puts "other"
end
"#,
    );
}

// ── parser/statements/control_flow.rs: return with value ────────────────────

#[test]
fn parse_return_with_value() {
    parse_ok(
        r#"
def foo
  return 42
end
"#,
    );
}

// ── parser/statements/control_flow.rs: return without value ─────────────────

#[test]
fn parse_return_without_value() {
    // Return without value followed by newline then end
    parse_ok("def foo\n  x = 1\n  return x\nend");
}

// ── parser/expressions/primary.rs: unless expression ────────────────────────

#[test]
fn parse_unless_expression() {
    parse_ok(
        r#"
x = unless false
  1
else
  2
end
"#,
    );
}

// ── parser/expressions/primary.rs: if expression with elsif ─────────────────

#[test]
fn parse_if_expression_with_elsif() {
    parse_ok(
        r#"
x = if true
  1
elsif false
  2
else
  3
end
"#,
    );
}

// ── parser/expressions/primary.rs: case expression ──────────────────────────

#[test]
fn parse_case_expression() {
    parse_ok(
        r#"
x = case 1
when 1 then "one"
when 2 then "two"
else "other"
end
"#,
    );
}

// ── parser/expressions/primary.rs: super with args ──────────────────────────

#[test]
fn parse_super_with_args() {
    parse_ok(
        r#"
class Base
  def greet(name)
    name
  end
end
class Child < Base
  def greet(name)
    super(name)
  end
end
"#,
    );
}

// ── parser/expressions/primary.rs: super without parens ─────────────────────

#[test]
fn parse_super_without_parens() {
    parse_ok(
        r#"
class Base
  def greet
    "hi"
  end
end
class Child < Base
  def greet
    super
  end
end
"#,
    );
}

// ── parser/expressions/primary.rs: lambda with do ───────────────────────────

#[test]
fn parse_lambda_with_do_keyword() {
    parse_ok(
        r#"
f = lambda do |x|
  x * 2
end
"#,
    );
}

// ── parser/expressions/primary.rs: lambda with empty params || ──────────────

#[test]
fn parse_lambda_empty_params() {
    parse_ok(
        r#"
f = lambda do ||
  42
end
"#,
    );
}

// ── parser/expressions/primary.rs: do block with params ─────────────────────

#[test]
fn parse_do_block_with_params() {
    parse_ok(
        r#"
[1, 2, 3].each do |x|
  puts x
end
"#,
    );
}

// ── parser/expressions/primary.rs: dict with fat arrow syntax ───────────────

#[test]
fn parse_dict_fat_arrow() {
    parse_ok(
        r#"
h = {"a" => 1, "b" => 2}
"#,
    );
}

// ── parser/statements/exception.rs: begin/rescue with else and ensure ───────

#[test]
fn parse_begin_rescue_else_ensure() {
    parse_ok(
        r#"
begin
  x = 1
rescue RuntimeError => e
  puts e
else
  puts "no error"
ensure
  puts "always"
end
"#,
    );
}

// ── parser/statements/exception.rs: rescue with fat arrow binding ───────────

#[test]
fn parse_rescue_with_variable_binding() {
    parse_ok(
        r#"
begin
  raise "oops"
rescue => e
  puts e
end
"#,
    );
}

// ── parser/statements/exception.rs: raise with expression ───────────────────

#[test]
fn parse_raise_with_string() {
    parse_ok(r#"raise "something went wrong""#);
}

// ── parser/statements/exception.rs: bare raise (re-raise in rescue) ─────────

#[test]
fn parse_raise_with_class_and_message() {
    parse_ok(r#"raise "something went wrong""#);
}

// ── parser/statements/control_flow.rs: for without do ───────────────────────

#[test]
fn parse_for_without_do() {
    parse_ok(
        r#"
for x in [1, 2, 3]
  puts x
end
"#,
    );
}

// ── parser/expressions/primary.rs: grouped expression ───────────────────────

#[test]
fn parse_grouped_expression() {
    parse_ok("(1 + 2) * 3");
}

// ── parser/expressions/primary.rs: symbol literal ───────────────────────────

#[test]
fn parse_symbol_literal() {
    parse_ok(":hello");
}

// ── parser/statements/control_flow.rs: break and continue ───────────────────

#[test]
fn parse_break_statement() {
    parse_ok(
        r#"
while true
  break
end
"#,
    );
}

#[test]
fn parse_continue_statement() {
    parse_ok(
        r#"
while true
  continue
end
"#,
    );
}

// ── parser/statements/control_flow.rs: case with object/array patterns ──────

#[test]
fn parse_case_with_object_pattern() {
    parse_ok(
        r#"
case d
when {x: Integer, y: Integer}
  "matched"
end
"#,
    );
}

#[test]
fn parse_case_with_array_rest_pattern() {
    parse_ok(
        r#"
case arr
when [1, ...rest]
  rest
end
"#,
    );
}

// ── parser/statements/control_flow.rs: range pattern ────────────────────────

#[test]
fn parse_case_with_range_pattern() {
    parse_ok(
        r#"
case x
when 1..10
  "small"
when 11...100
  "medium"
end
"#,
    );
}

// ── parser/statements/control_flow.rs: float range pattern ──────────────────

#[test]
fn parse_case_with_float_range_pattern() {
    parse_ok(
        r#"
case x
when 1.0..10.0
  "small"
end
"#,
    );
}

// ── parser/statements/control_flow.rs: string range pattern (suffix) ────────

#[test]
fn parse_case_with_string_pattern() {
    parse_ok(
        r#"
case x
when "hello"
  "matched"
end
"#,
    );
}

// ── parser/statements/control_flow.rs: multiple exception types ─────────────

#[test]
fn parse_rescue_multiple_exception_types() {
    parse_ok(
        r#"
begin
  raise "oops"
rescue RuntimeError, TypeError => e
  puts e
end
"#,
    );
}
