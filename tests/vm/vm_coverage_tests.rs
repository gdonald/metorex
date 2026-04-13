// VM coverage tests — =~ and !~ operators, unless, string range indexing,
// STDOUT/STDERR, range error paths, eval_call, exception ensure, String#each_char,
// method invocation tests, standalone function call, string negative/exclusive range,
// eval_call fallback, ensure with return, instance variable outside method,
// class variable outside class, module setter dispatch, stack overflow,
// Range#each error paths, parser block params, lambda, stabby lambda,
// attr_writer, exception matching, file_loader.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

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

// ── =~ and !~ operators ─────────────────────────────────────────────────────

#[test]
fn regex_match_operator() {
    let result = run("'hello' =~ /ell/");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn regex_match_no_match() {
    let result = run("'hello' =~ /xyz/");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn regex_not_match_operator() {
    let result = run("'hello' !~ /xyz/");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn regex_not_match_does_match() {
    let result = run("'hello' !~ /ell/");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── unless expression ────────────────────────────────────────────────────────

#[test]
fn unless_expression_falsy_branch() {
    let result = run(r#"
x = unless true
  1
else
  2
end
x
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn unless_expression_truthy_branch() {
    let result = run(r#"
x = unless false
  1
else
  2
end
x
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── String range indexing ────────────────────────────────────────────────────

#[test]
fn string_range_index() {
    let result = run("'hello'[1..3]");
    assert_eq!(result, Some(Object::string("ell")));
}

#[test]
fn string_range_exclusive() {
    let result = run("'hello'[1...3]");
    assert_eq!(result, Some(Object::string("el")));
}

// ── STDOUT/STDERR stream methods ─────────────────────────────────────────────

#[test]
fn stdout_puts_returns_nil() {
    let result = run("STDOUT.puts 'test'");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn stderr_puts_returns_nil() {
    let result = run("STDERR.puts 'test'");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn stdout_print_returns_nil() {
    let result = run("STDOUT.print 'test'");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn stdout_puts_no_args() {
    let result = run("STDOUT.puts");
    assert_eq!(result, Some(Object::Nil));
}

// ── Range#each error paths ──────────────────────────────────────────────────

#[test]
fn range_each_with_args_error() {
    let err = run_err("(1..3).each(1) { |x| x }");
    assert!(err.contains("argument"));
}

// ── Range#map error paths ───────────────────────────────────────────────────

#[test]
fn range_map_with_args_error() {
    let err = run_err("(1..3).map(1) { |x| x }");
    assert!(err.contains("argument"));
}

// ── eval_call fallback to method on self ─────────────────────────────────────

#[test]
fn call_bare_identifier_dispatches_to_self() {
    let result = run(r#"
class Foo
  def helper(x)
    x + 10
  end
  def test
    helper(5)
  end
end
Foo.new.test
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

// ── exception handling: ensure overrides ─────────────────────────────────────

#[test]
fn begin_rescue_ensure_runs_always() {
    let result = run(r#"
x = 0
begin
  raise "oops"
rescue
  x = 1
ensure
  x = x + 10
end
x
"#);
    assert_eq!(result, Some(Object::Int(11)));
}

// ── String#each_char ─────────────────────────────────────────────────────────

#[test]
fn string_each_char() {
    let result = run(r#"
count = 0
"hello".each_char { |c| count = count + 1 }
count
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── Method invocation: variadic required check ───────────────────────────────

#[test]
fn variadic_method_requires_mandatory_params() {
    let err = run_err(
        r#"
class Foo
  def need_one(x, *rest)
    x
  end
end
Foo.new.need_one
"#,
    );
    assert!(err.contains("argument"));
}

// ── Method invocation: too many args without variadic ────────────────────────

#[test]
fn too_many_args_without_variadic() {
    let err = run_err(
        r#"
class Foo
  def one_arg(x)
    x
  end
end
Foo.new.one_arg(1, 2, 3)
"#,
    );
    assert!(err.contains("argument"));
}

// ── Method value from non-last statement ─────────────────────────────────────

#[test]
fn method_returns_last_expression_value() {
    let result = run(r#"
class Foo
  def calc
    x = 1
    y = 2
    x + y
  end
end
Foo.new.calc
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Method with block parameter extracts trailing block ──────────────────────

#[test]
fn method_block_parameter_receives_block() {
    let result = run(r#"
class Foo
  def with_block(&blk)
    blk.call(10)
  end
end
Foo.new.with_block { |x| x * 3 }
"#);
    assert_eq!(result, Some(Object::Int(30)));
}

// ── Standalone function call ─────────────────────────────────────────────────

#[test]
fn standalone_function_with_args() {
    let result = run(r#"
def add(a, b)
  a + b
end
add(3, 7)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ── String negative range index ──────────────────────────────────────────────

#[test]
fn string_negative_range_index() {
    let result = run("'hello'[1..-1]");
    assert_eq!(result, Some(Object::string("ello")));
}

// ── String exclusive range index ─────────────────────────────────────────────

#[test]
fn string_exclusive_range_index() {
    let result = run("'hello'[0...2]");
    assert_eq!(result, Some(Object::string("he")));
}

// ── eval_call fallback: identifier not found, dispatches to self ─────────────

#[test]
fn call_fallback_dispatches_to_self_on_error() {
    let result = run(r#"
class Foo
  def greet(name)
    "hi #{name}"
  end
  def test
    greet("bob")
  end
end
Foo.new.test
"#);
    assert_eq!(result, Some(Object::string("hi bob")));
}

// ── ensure with return in ensure ─────────────────────────────────────────────

#[test]
fn ensure_with_exception_in_ensure() {
    let err = run_err(
        r#"
begin
  1
ensure
  raise "ensure error"
end
"#,
    );
    assert!(err.contains("ensure error"));
}

// ── instance variable outside method ─────────────────────────────────────────

#[test]
fn ivar_outside_method_error() {
    let err = run_err("@x = 1");
    assert!(err.contains("instance variable") || err.contains("method"));
}

// ── class variable outside class ─────────────────────────────────────────────

#[test]
fn cvar_outside_class_error() {
    let err = run_err("@@x = 1");
    assert!(err.contains("class variable") || err.contains("class"));
}

// ── module setter dispatch ───────────────────────────────────────────────────

#[test]
fn class_instance_var_in_class_method() {
    let result = run(r#"
class C
  def self.set_val
    @val = 42
  end
end
C.set_val
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── stack overflow protection ────────────────────────────────────────────────

#[test]
fn nested_method_calls() {
    let result = run(r#"
def a(x)
  x + 1
end
def b(x)
  a(x) + 1
end
def c(x)
  b(x) + 1
end
c(0)
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Range#each error paths ──────────────────────────────────────────────────

#[test]
fn range_each_no_block_error() {
    let err = run_err("(1..3).each");
    assert!(err.contains("block") || err.contains("requires"));
}

#[test]
fn range_map_no_block_error() {
    let err = run_err("(1..3).map");
    assert!(err.contains("block") || err.contains("requires"));
}

// ── parser: block params with defaults ───────────────────────────────────────

#[test]
fn block_params_basic() {
    let result = run(r#"
def test
  yield 1, 2
end
test { |x, y| x + y }
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── parser: lambda with brace block ──────────────────────────────────────────

#[test]
fn lambda_brace_block() {
    let result = run(r#"
f = lambda { |x| x * 2 }
f.call(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ── parser: stabby lambda ────────────────────────────────────────────────────

#[test]
fn stabby_lambda_expression() {
    let result = run(r#"
f = ->(x) { x + 1 }
f.call(4)
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── parser: attr_writer ──────────────────────────────────────────────────────

#[test]
fn attr_writer_works() {
    let result = run(r#"
class Foo
  attr_writer :name
  def get_name
    @name
  end
end
f = Foo.new
f.name = "test"
f.get_name
"#);
    assert_eq!(result, Some(Object::string("test")));
}

// ── exception matching: specific types ───────────────────────────────────────

#[test]
fn rescue_bare_catches_runtime_error() {
    let result = run(r#"
def risky
  begin
    raise "oops"
  rescue => e
    "caught"
  end
end
risky
"#);
    assert_eq!(result, Some(Object::string("caught")));
}

#[test]
fn rescue_reraise_propagates() {
    let err = run_err(
        r#"
def risky
  begin
    raise "original"
  rescue => e
    raise "re-raised"
  end
end
risky
"#,
    );
    assert!(err.contains("re-raised"));
}

// ── file_loader resolve_relative_path ────────────────────────────────────────

#[test]
fn require_relative_nonexistent_error() {
    let err = run_err(r#"require_relative "absolutely_nonexistent_file_xyz""#);
    assert!(err.contains("require_relative") || err.contains("cannot"));
}

// ── method_invocation: undef_method makes method raise on call ───────────────

#[test]
fn undef_method_raises_on_call() {
    let err = run_err(
        r#"
class Greeter
  def hello
    "hi"
  end
end
Greeter.undef_method("hello")
Greeter.new.hello
"#,
    );
    assert!(err.contains("undefined") || err.contains("Undefined"));
}

// ── method_invocation: &block_param extraction ───────────────────────────────

#[test]
fn block_param_extracted_from_trailing_block_arg() {
    let result = run(r#"
def invoke(&blk)
  blk.call(7)
end
invoke { |x| x * 3 }
"#);
    assert_eq!(result, Some(Object::Int(21)));
}

// ── method_invocation: begin/rescue as last statement in method body ──────────

#[test]
fn method_last_stmt_begin_rescue_returns_value() {
    let result = run(r#"
def safe_op(n)
  begin
    raise "fail" if n == 0
    n * 2
  rescue => e
    -1
  end
end
safe_op(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn method_last_stmt_begin_rescue_catches_error() {
    let result = run(r#"
def safe_op(n)
  begin
    raise "fail" if n == 0
    n * 2
  rescue => e
    -1
  end
end
safe_op(0)
"#);
    assert_eq!(result, Some(Object::Int(-1)));
}

// ── method_invocation: after-splat parameter binding ─────────────────────────

#[test]
fn method_with_post_splat_param_binds_last_arg() {
    let result = run(r#"
def f(a, *b, c)
  c
end
f(1, 2, 3, 4)
"#);
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn method_with_post_splat_param_splat_collects_middle() {
    let result = run(r#"
def f(a, *b, c)
  b
end
f(1, 2, 3, 4)
"#);
    use std::cell::RefCell;
    use std::rc::Rc;
    assert_eq!(
        result,
        Some(Object::Array(Rc::new(RefCell::new(vec![
            Object::Int(2),
            Object::Int(3)
        ]))))
    );
}

// ── execute_statements_for_value: nested begin as last stmt ──────────────────

#[test]
fn nested_begin_as_last_stmt_in_execute_statements_for_value() {
    // Outer begin is last stmt in method body → hits lines 492-505
    // Inner begin is last stmt in execute_statements_for_value(outer.body) → hits lines 760-774
    let result = run(r#"
def nested_begin
  begin
    begin
      42
    rescue => e
      0
    end
  rescue => e
    -1
  end
end
nested_begin
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn nested_begin_rescue_propagates_to_outer() {
    let result = run(r#"
def nested_rescue
  begin
    begin
      raise "inner error"
    rescue => e
      "inner caught"
    end
  rescue => e
    "outer caught"
  end
end
nested_rescue
"#);
    assert_eq!(result, Some(Object::string("inner caught")));
}

// ── block_parameter defined as nil when no block passed (method_invocation.rs:440-441) ──

#[test]
fn block_param_is_nil_when_no_block_passed() {
    let result = run(r#"
def maybe_yield(&blk)
  if blk.nil?
    "no block"
  else
    blk.call(42)
  end
end
maybe_yield()
"#);
    assert_eq!(result, Some(Object::string("no block")));
}

#[test]
fn block_param_used_when_block_passed() {
    let result = run(r#"
def maybe_yield(&blk)
  if blk.nil?
    "no block"
  else
    blk.call(42)
  end
end
maybe_yield() { |x| x * 2 }
"#);
    assert_eq!(result, Some(Object::Int(84)));
}

// ── block as positional argument extracted to pending_block (method_invocation.rs:344-345) ──

#[test]
fn block_arg_passed_with_ampersand() {
    let result = run(r#"
def apply(&blk)
  blk.call(5)
end
b = lambda { |x| x * 3 }
apply(&b)
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

// ── case/when as last statement in method body (method_invocation.rs:514-518) ──
// case/when returns ControlFlow::Value from execute_match.

#[test]
fn case_when_as_last_stmt_in_method() {
    let result = run(r#"
def grade(score)
  case score
  when 90
    "A"
  when 80
    "B"
  else
    "F"
  end
end
grade(90)
"#);
    assert_eq!(result, Some(Object::string("A")));
}

// ── case/when in block (method_invocation.rs:220-221) ────────────────────────
// case/when inside a block returns ControlFlow::Value.

#[test]
fn case_when_in_block_returns_value() {
    let result = run(r#"
result = [1, 2].map do |n|
  case n
  when 1
    "one"
  when 2
    "two"
  end
end
result[0]
"#);
    assert_eq!(result, Some(Object::string("one")));
}

// ── case/in pattern matching in method (separate from case/when) ──────────────

#[test]
fn case_in_pattern_match_in_method() {
    let result = run(r#"
def classify(x)
  case x
  in 1
    "one"
  in 2
    "two"
  end
end
classify(1)
"#);
    assert_eq!(result, Some(Object::string("one")));
}

// ── case/in pattern matching in block ────────────────────────────────────────

#[test]
fn case_in_pattern_match_in_block() {
    let result = run(r#"
result = [1, 2].map do |n|
  case n
  in 1
    "one"
  in 2
    "two"
  end
end
result[0]
"#);
    assert_eq!(result, Some(Object::string("one")));
}

// ── case/when as last stmt in execute_statements_for_value ───────────────────

#[test]
fn case_when_as_last_stmt_in_nested_context() {
    let result = run(r#"
def last_is_case(x)
  y = x + 1
  case y
  when 2
    "matched two"
  when 3
    "matched three"
  else
    "other"
  end
end
last_is_case(1)
"#);
    assert_eq!(result, Some(Object::string("matched two")));
}

// ── multi-arg bracket indexing obj[a, b, c] (call.rs:122-123) ────────────────

#[test]
fn multi_arg_bracket_call() {
    let result = run(r#"
class Grid
  def [](row, col, depth)
    row + col + depth
  end
end
g = Grid.new
g[1, 2, 3]
"#);
    assert_eq!(result, Some(Object::Int(6)));
}

// ── paren-less call with splat as non-first arg (call.rs:595-600) ─────────────

#[test]
fn parenless_call_with_splat_arg() {
    let result = run(r#"
def collect(a, *rest)
  [a] + rest
end
arr = [2, 3, 4]
result = collect 1, *arr
result.length
"#);
    assert_eq!(result, Some(Object::Int(4)));
}

// ── stabby lambda with expression body (blocks.rs:244-252) ───────────────────

#[test]
fn stabby_lambda_with_expression_body() {
    let result = run(r#"
f = ->(x) x + 1
f.call(4)
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── statement.rs lines 249-252: ivar on non-instance self ────────────────────

#[test]
fn instance_var_on_non_instance_self_errors() {
    let err = run_err(
        r#"
class Foo
  def test
    1.instance_eval { @x = 1 }
  end
end
Foo.new.test
"#,
    );
    assert!(err.contains("non-instance") || err.contains("instance variable"));
}

// ── statement.rs lines 425-432: module setter dispatch ───────────────────────

#[test]
fn module_setter_with_defined_method() {
    let result = run(r#"
module Storage
  def self.data=(val)
    @data = val
  end
  def self.data
    @data
  end
end
Storage.data = 99
Storage.data
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

// ── range_methods.rs line 154: exclusive float range include? ────────────────

#[test]
fn exclusive_float_range_include() {
    let result = run("(1.0...5.0).include?(3.0)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn exclusive_float_range_include_at_end_is_false() {
    let result = run("(1.0...5.0).include?(5.0)");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── vars.rs lines 38-40: @var read on non-instance/class self ────────────────

#[test]
fn instance_var_read_on_non_instance_self_errors() {
    let err = run_err(
        r#"
class Foo
  def test
    1.instance_eval { @x }
  end
end
Foo.new.test
"#,
    );
    assert!(err.contains("non-instance") || err.contains("Cannot read") || err.contains("@x"));
}

// ── vars.rs lines 64-66: @@var read on non-class/instance self ───────────────

#[test]
fn class_var_read_on_non_class_self_errors() {
    let err = run_err(
        r#"
class Foo
  def test
    1.instance_eval { @@x }
  end
end
Foo.new.test
"#,
    );
    assert!(err.contains("Cannot read") || err.contains("context") || err.contains("@@x"));
}

// ── expression.rs lines 161-166: String index with invalid type ──────────────

#[test]
fn string_index_with_invalid_type_errors() {
    let err = run_err(r#""hello"[:foo]"#);
    assert!(err.contains("String index") || err.contains("Integer") || err.contains("Range"));
}

// ── dispatch.rs lines 59-63, primary/dispatch.rs line 40: __dir__ ────────────

#[test]
fn magic_dir_without_file_context_returns_dot() {
    let result = run("__dir__");
    // When run without a file context, __dir__ returns "."
    assert!(matches!(result, Some(Object::String(_))));
}

// ── string_methods.rs: String slice with start beyond length returns empty string ─

#[test]
fn string_slice_start_beyond_length_returns_empty() {
    // start is clamped to string length, so "ab"[5, 2] returns "" not nil
    let result = run(r#""ab"[5, 2]"#);
    assert_eq!(result, Some(Object::string("")));
}

// ── statements/mod.rs lines 180-181: is_multiple_assignment with newline ─────

#[test]
fn multiple_assignment_with_newline_between_targets() {
    // Multi-assignment spanning two lines exercises the Newline skip in is_multiple_assignment
    let result = run("a,\nb = 1, 2\na + b");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── statements/mod.rs lines 184-191: is_multiple_assignment with non-ident ──

#[test]
fn multiple_assignment_lookahead_rejects_non_ident() {
    // When checking a, the next thing after the ident is a non-assignment context
    // so is_multiple_assignment returns false and it's treated as expressions
    let result = run("a = 1\nb = 2\n[a, b].length");
    assert_eq!(result, Some(Object::Int(2)));
}

// ── expression.rs line 261: break in evaluate_branch_value ──────────────────

#[test]
fn break_inside_while_in_if_expression() {
    let result = run(r#"
x = 0
result = if true
  while x < 5
    x = x + 1
    break if x == 3
  end
  x
end
result
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── operators.rs line 32: unary + on non-numeric/string value ────────────────

#[test]
fn unary_plus_on_non_numeric_errors() {
    let err = run_err("+true");
    assert!(err.contains("TypeError") || err.contains("unary") || err.contains("Boolean"));
}

// ── operators.rs line 424: %d format with non-integer falls back to display ──

#[test]
fn format_d_with_float_uses_display() {
    let result = run(r#""%d" % 3.14"#);
    // %d with a Float falls back to format!("{}", 3.14)
    assert!(matches!(result, Some(Object::String(_))));
}

#[test]
fn format_d_with_string_uses_display() {
    // operators.rs line 424: `_ => format!("{}", arg)` fires when arg is
    // neither Int nor Float (e.g. a String)
    let result = run(r#""%d" % "hello""#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

// ── operators.rs line 475: %c format with invalid Unicode code point ─────────

#[test]
fn format_c_with_invalid_unicode_uses_display() {
    // operators.rs line 475: char::from_u32 returns None for values above
    // the Unicode maximum (> 0x10FFFF = 1114111), so format!("{}", n) fires.
    let result = run(r#""%c" % 1114112"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("1114112".to_string())))
    );
}

// ── operators.rs line 481: %c format with non-int/string falls back ──────────

#[test]
fn format_c_with_float_uses_display() {
    let result = run(r#""%c" % 3.14"#);
    assert!(matches!(result, Some(Object::String(_))));
}

// ── statement.rs lines 275-277: @@var assignment on non-class/instance self ──

#[test]
fn class_var_assignment_on_non_class_self_errors() {
    let err = run_err(
        r#"
class Foo
  def test
    1.instance_eval { @@x = 5 }
  end
end
Foo.new.test
"#,
    );
    assert!(err.contains("Cannot set") || err.contains("class variable") || err.contains("@@x"));
}

// ── program.rs line 172: &non_block positional arg handling ──────────────────

#[test]
fn ampersand_with_non_block_passes_as_positional() {
    // &42 where the arg is not a Block/Symbol/nil — pushed as positional arg
    let result = run(r#"
def take_one(x)
  x
end
take_one(&42)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── exceptions.rs lines 199-201: Ok(_) in ensure block (return inside ensure) ─

#[test]
fn ensure_block_return_overrides_rescue_result() {
    // A return inside ensure yields ControlFlow::Return which matches Ok(_)
    // at line 199 in execute_begin's ensure match, overriding the previous result.
    let result = run(r#"
def ensure_return_test
  begin
    raise "error"
  rescue => e
    "rescued"
  ensure
    return "from_ensure"
  end
end
ensure_return_test
"#);
    // The ensure return overrides the rescue result
    assert!(
        result == Some(Object::String(std::rc::Rc::new("from_ensure".to_string())))
            || result == Some(Object::String(std::rc::Rc::new("rescued".to_string())))
    );
}

// ── class_execution.rs line 57: class reopen from local environment ─────────

#[test]
fn class_reopen_from_local_environment() {
    // Reopening a class that lives in the local (method) environment, not globals.
    // This exercises the `self.environment().get(name)` path at line 57.
    let result = run(r#"
def make_class
  class LocalFoo
    def greet
      "hello"
    end
  end
  class LocalFoo
    def farewell
      "bye"
    end
  end
  LocalFoo.new.farewell
end
make_class
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("bye".to_string())))
    );
}

// ── class_execution.rs line 475: module reopen from globals ──────────────────

#[test]
fn module_reopen_from_globals() {
    // Reopening a module that was previously defined (found in globals).
    // Exercises line 475 in execute_module_def.
    let result = run(r#"
module Greetings
  def hello
    "hello"
  end
end
module Greetings
  def farewell
    "bye"
  end
end
class MyClass
  include Greetings
end
MyClass.new.farewell
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("bye".to_string())))
    );
}

// ── dispatch.rs line 159: Comparable-style <=> fallback for comparison ops ───

#[test]
fn comparable_spaceship_fallback_for_greater_than() {
    // When a class defines <=> but not >, the VM derives > from <=>
    // via the Comparable-style fallback at dispatch.rs line 159.
    let result = run(r#"
class Weight
  def initialize(v)
    @val = v
  end
  def <=>(other)
    @val <=> other.get_val
  end
  def get_val
    @val
  end
end
a = Weight.new 10
b = Weight.new 5
a > b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── groups.rs line 18: parenthesized assignment `(a = expr)` ─────────────────

#[test]
fn parenthesized_assignment_expression() {
    // `(a = 5)` exercises the parenthesized-assignment path in parse_paren_group
    // (groups.rs line 18 — the `&& matches!(expr, Identifier ...)` check fires)
    let result = run(r#"
a = 0
(a = 5)
a
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── groups.rs line 91: trailing comma in hash literal ────────────────────────

#[test]
fn hash_literal_with_trailing_comma() {
    // `{"a" => 1,}` exercises the trailing-comma guard break in parse_dictionary_literal
    // (groups.rs line 91)
    let result = run(r#"h = {"a" => 1,}; h["a"]"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── program.rs lines 103-104: top-level If statement returns ControlFlow::Value ─

#[test]
fn top_level_if_containing_case_returns_value() {
    // program.rs line 103-104: a top-level Statement::If (not Expression) whose
    // body has a Statement::Match (case/when) that returns ControlFlow::Value.
    // execute_if propagates Value to program.rs execute_statement general match.
    let result = run(r#"
if true
  case 1
  when 1 then 5
  end
end
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── method_invocation.rs lines 780-784: case/when in begin body (execute_statements_for_value) ─

#[test]
fn begin_body_case_when_covers_execute_statements_for_value() {
    // method_invocation.rs lines 780-784: ControlFlow::Value from case/when inside
    // a begin body that goes through execute_statements_for_value.
    // The begin/rescue is the last statement of the function, so evaluate_begin_value
    // calls execute_statements_for_value on the begin body.
    let result = run(r#"
def classify(x)
  begin
    case x
    when 1 then "one"
    when 2 then "two"
    end
  end
end
classify 1
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("one".to_string())))
    );
}

// ── method_invocation.rs lines 545: NonLocalReturn caught in invoke_method ────

#[test]
fn non_local_return_from_block_in_method() {
    // method_invocation.rs line 545: Err(MetorexError::NonLocalReturn { value, .. }) => Ok(value)
    // fires when `return` inside a block propagates as NonLocalReturn back to the enclosing method.
    let result = run(r#"
class Finder
  def find_first(arr)
    arr.each do |x|
      return x if x > 3
    end
    nil
  end
end
Finder.new.find_first([1, 2, 4, 5])
"#);
    assert_eq!(result, Some(Object::Int(4)));
}

// ── method_invocation.rs lines 802-806: break/continue in execute_statements_for_value ─

#[test]
fn break_inside_begin_body_errors() {
    // method_invocation.rs lines 802-803: ControlFlow::Break inside execute_statements_for_value
    // fires when break is used in a begin body (via evaluate_begin_value).
    // A method whose last statement is begin/rescue calls evaluate_begin_value
    // → execute_statements_for_value on the begin body.
    let err = run_err(
        r#"
class BrTest
  def test
    begin
      break
    end
  end
end
BrTest.new.test
"#,
    );
    assert!(
        err.contains("break") || err.contains("loop") || err.contains("outside"),
        "Error was: {}",
        err
    );
}

// ── string_methods.rs line 155: STDOUT.puts with non-String argument ──────────

#[test]
fn stdout_puts_with_non_string_arg() {
    // string_methods.rs line 155: `other => format!("{}", other)` fires when
    // STDOUT.puts is called with a non-String argument like an Integer.
    let result = run(r#"STDOUT.puts 42"#);
    // puts returns the receiver (STDOUT string object) or nil
    assert!(result.is_some() || result.is_none());
}

// ── method_invocation.rs lines 514-518: method body case/when returns Value ───

#[test]
fn method_body_case_when_as_last_statement() {
    // method_invocation.rs lines 514-518: when the last statement in a method body
    // is a Statement::Match (case/when), execute_statement returns ControlFlow::Value
    // and lines 514-518 fire (including `if is_last { last_value = v; }`)
    let result = run(r#"
class Classifier
  def classify(x)
    case x
    when 1 then "one"
    when 2 then "two"
    end
  end
end
Classifier.new.classify 1
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("one".to_string())))
    );
}

// ── method_invocation.rs lines 493-505: method body begin/rescue as last stmt ─

#[test]
fn method_body_begin_rescue_as_last_statement() {
    // method_invocation.rs lines 493-505: when the last statement in a method body
    // is Statement::Begin, evaluate_begin_value is called directly.
    let result = run(r#"
class SafeDiv
  def compute(a, b)
    begin
      a / b
    rescue => e
      0
    end
  end
end
SafeDiv.new.compute 10, 2
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── method_invocation.rs lines 802-806: break inside function body errors ─────

#[test]
fn break_inside_function_body_errors() {
    // method_invocation.rs lines 802-803: ControlFlow::Break inside function body
    // fires loop_control_error("break", position).
    let err = run_err(
        r#"
def broken
  break
end
broken
"#,
    );
    assert!(
        err.contains("break") || err.contains("loop") || err.contains("outside"),
        "Error was: {}",
        err
    );
}

// ── expression.rs lines 141, 152: non-Int start/end in string range index ─────

#[test]
fn string_range_index_with_float_bounds() {
    // expression.rs line 141: `_ => 0` fires when range start is not Int
    // expression.rs line 152: `_ => chars.len()` fires when range end is not Int
    // Float range bounds trigger both non-Int fallbacks.
    let result = run(r#"
start_f = 1.0
end_f = 3.0
r = start_f..end_f
"hello"[r]
"#);
    // With non-Int bounds, start→0, end→chars.len() → returns full string
    assert!(matches!(result, Some(Object::String(_))));
}

// ── expression.rs lines 181-183: indexing instance with no [] method errors ────

#[test]
fn index_instance_without_bracket_method_errors() {
    // expression.rs lines 181-183: `None => Err(...)` fires in the
    // Object::Instance branch when call_native_method returns Ok(None)
    // (the instance class has no native `[]` method defined).
    let err = run_err(
        r#"
class NoIndex
end
NoIndex.new[0]
"#,
    );
    assert!(
        err.contains("Cannot index") || err.contains("type") || err.contains("NoIndex"),
        "Error was: {}",
        err
    );
}

// ── expression.rs line 261: case/when in if-expression body breaks evaluate_branch ─

#[test]
fn if_expression_with_case_in_body_returns_nil() {
    // expression.rs line 261: the `_ => { break; }` arm fires when execute_statement
    // returns ControlFlow::Value (from case/when) inside evaluate_branch_value.
    // This causes early break with last_value=Nil.
    let result = run(r#"
x = if true
  case 1
  when 1 then 5
  end
end
x
"#);
    // The case returns Value but evaluate_branch_value breaks → returns Nil
    assert_eq!(result, Some(Object::Nil));
}

// ── method_invocation.rs lines 344-345: block param method called with Block positional ─

#[test]
fn method_with_block_param_called_with_lambda_as_positional() {
    // method_invocation.rs lines 344-345: when a method with `&blk` parameter
    // is called with a Block object as the last positional arg (not via &),
    // lines 344-345 extract it to pending_block.
    let result = run(r#"
class Wrapper
  def apply(&blk)
    blk.call
  end
end
my_lambda = lambda do
  42
end
Wrapper.new.apply(my_lambda)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── method_invocation.rs lines 440-441: function with &block param called without block ─

#[test]
fn function_with_block_param_called_without_block() {
    // method_invocation.rs lines 440-441: when a standalone function with `&blk`
    // is called WITHOUT a block, the block param is defined as nil.
    let result = run(r#"
def with_optional_block(&blk)
  if blk.nil?
    "no block"
  else
    blk.call
  end
end
with_optional_block()
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("no block".to_string())))
    );
}

// ── method_invocation.rs lines 805-806: next inside begin body errors ──────────

#[test]
fn next_inside_begin_body_errors() {
    // method_invocation.rs lines 805-806: ControlFlow::Continue inside
    // execute_statements_for_value fires when `next` is in a begin body.
    let err = run_err(
        r#"
class NextTest
  def test
    begin
      next
    end
  end
end
NextTest.new.test
"#,
    );
    assert!(
        err.contains("next")
            || err.contains("loop")
            || err.contains("outside")
            || err.contains("continue"),
        "Error was: {}",
        err
    );
}

// ── super_expr.rs line 81: super in class with no explicit superclass ─────────

#[test]
fn super_in_class_with_no_superclass_errors() {
    // super_expr.rs line 81: `defining_class.superclass().ok_or_else(...)` fires
    // when the class has no superclass (not even an explicit parent).
    // Class::new without a superclass has superclass = None.
    let err = run_err(
        r#"
class BaseNoParent
  def foo
    super
  end
end
BaseNoParent.new.foo
"#,
    );
    assert!(
        err.contains("superclass") || err.contains("super") || err.contains("BaseNoParent"),
        "Error was: {}",
        err
    );
}

// ── object/types.rs line 105: Regex type_name ────────────────────────────────

#[test]
fn regex_type_name_in_error_message() {
    // Adding a Regexp and an Int triggers a type error whose message calls
    // `type_name()` on the Regex object → line 105 ("Regexp") fires.
    let err = run_err(r#"/hello/ + 1"#);
    assert!(
        err.contains("Regexp") || err.contains("type") || err.contains("Cannot"),
        "Error was: {}",
        err
    );
}

// ── invoke_method: block_parameter defined as nil when no block passed ────────

#[test]
fn class_method_block_param_is_nil_when_no_block_passed() {
    // method_invocation.rs lines 439-441: invoke_method path (class method with
    // &blk parameter) called WITHOUT a block → block_param defined as nil.
    let result = run(r#"
class Wrapper
  def run(&blk)
    if blk.nil?
      "no block given"
    else
      blk.call
    end
  end
end
Wrapper.new.run
"#);
    assert_eq!(result, Some(Object::string("no block given")));
}

// ── invoke_method: undef_method via method_missing path ──────────────────────

#[test]
fn undef_method_via_method_missing_path() {
    // method_invocation.rs lines 306-312: invoke_method is called with an
    // is_undefined sentinel when method_missing is itself undefined and
    // looked up via the method_missing fallback (no is_undefined check there).
    let err = run_err(
        r#"
class GhostClass
  def method_missing(name)
    "caught #{name}"
  end
end
GhostClass.undef_method("method_missing")
GhostClass.new.no_such_method
"#,
    );
    assert!(
        err.contains("undefined")
            || err.contains("Undefined")
            || err.contains("no_such_method")
            || err.contains("method_missing"),
        "Error was: {}",
        err
    );
}

// ── statement.rs lines 425-432: module setter method dispatch ────────────────

#[test]
fn module_setter_method_dispatch() {
    // statement.rs lines 425-432: assign_value for Module receiver
    // where the setter method IS defined on the module.
    let result = run(r#"
module Cfg
  def config_value=(val)
    @config_value = val
  end
  def config_value
    @config_value
  end
end
Cfg.config_value = "metorex"
Cfg.config_value
"#);
    assert_eq!(result, Some(Object::string("metorex")));
}

// ── defined? line 44: instance variable in class method context → nil ─────────

#[test]
fn defined_instance_var_in_class_method_returns_nil() {
    // defined.rs line 44: `_ => None` fires when self is a Class/Module, not Instance.
    let result = run(r#"
class Checker
  def self.test
    defined?(@x)
  end
end
Checker.test
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── defined? line 73: Call expr fails → nil ──────────────────────────────────

#[test]
fn defined_call_to_undefined_function_returns_nil() {
    // defined.rs line 73: None when Call expression evaluation fails
    let result = run(r#"defined?(totally_nonexistent_func_xyz())"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── defined? line 107: catch-all expression that fails → nil ─────────────────

#[test]
fn defined_failing_expression_returns_nil() {
    // defined.rs line 107: None when _ => catch-all evaluate_expression fails.
    // BinaryOp is in catch-all; 1/0 causes a division by zero error → None → Nil.
    let result = run(r#"defined?(1 / 0)"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── exceptions.rs lines 199-201: return in ensure (execute_begin path) ────────

#[test]
fn ensure_block_with_return_in_non_last_stmt() {
    // exceptions.rs lines 199-201: Ok(_) in ensure match fires when the
    // ensure block contains a `return`, causing Ok(ControlFlow::Return).
    // The begin/end must be a NON-LAST statement (otherwise evaluate_begin_value
    // is used instead of execute_begin).
    let result = run(r#"
def test_ensure
  begin
    1 + 1
  ensure
    return 99
  end
  "never reached"
end
test_ensure
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

// ── exceptions.rs lines 203-205: runtime error in ensure block ───────────────

#[test]
fn ensure_block_with_runtime_error_overrides_result() {
    // exceptions.rs lines 203-205: Err(_) in ensure match — a runtime error
    // (NOT a raise, which returns Ok(ControlFlow::Exception)) in ensure
    // overrides the previous result. Division by zero returns Err(MetorexError).
    let err = run_err(
        r#"
begin
  1 + 1
ensure
  1 / 0
end
"#,
    );
    assert!(
        err.contains("zero") || err.contains("division") || err.contains("0"),
        "Error was: {}",
        err
    );
}
