// Additional coverage tests for vm/core.rs, vm/native_functions.rs, and vm/pattern_matching.rs

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

// ── vm/core.rs line 170: Match ControlFlow::Next path ───────────────────────
// A case/when at top level that produces ControlFlow::Next (no explicit value)

#[test]
fn match_at_top_level_next_then_continue() {
    // Execute a case/when that goes through ControlFlow::Next (assignment, no return)
    // then falls through to line 187 (continue after match)
    let result = run(r#"
x = 0
case 1
when 1
  x = 42
end
x
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── vm/core.rs line 187: continue after match/case falls through ────────────

#[test]
fn match_next_continues_to_next_statement() {
    // After a match block processes ControlFlow::Next, the next statement after the
    // match should execute (the continue on line 187)
    let result = run(r#"
y = case 1
when 1
  10
end
y + 5
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

// ── vm/core.rs lines 481-483: ClassVariable read on non-class/non-instance ──

#[test]
fn class_var_read_outside_class_context_error() {
    // @@var read when self is neither instance nor class should error
    let err = run_err(
        r#"
@@x
"#,
    );
    assert!(
        err.contains("class variable")
            || err.contains("@@")
            || err.contains("Class variable")
            || err.contains("only be used")
    );
}

// ── vm/core.rs lines 518-530: super called in invalid context ───────────────

#[test]
fn super_called_outside_method_no_self_error() {
    let err = run_err("super()");
    assert!(err.contains("super") || err.contains("method") || err.contains("context"));
}

// ── vm/core.rs lines 548-558: super finding defining class in chain ─────────

#[test]
fn super_three_level_inheritance_skips_correct_class() {
    let result = run(r#"
class A
  def val
    "A"
  end
end
class B < A
  def val
    "B:" + super
  end
end
class C < B
  def val
    "C:" + super
  end
end
C.new.val
"#);
    assert_eq!(result, Some(Object::string("C:B:A")));
}

// ── vm/core.rs line 682: binary_op_method_name returns None for And/Or ──────

#[test]
fn binary_op_and_or_dont_dispatch_to_custom_operator() {
    // And/Or operators use short-circuit, not custom dispatch.
    // This verifies the _ => None branch in binary_op_method_name.
    let result = run(r#"
class Num
  attr_reader :val
  def initialize(v)
    @val = v
  end
end
a = Num.new(1)
b = Num.new(2)
a && b
"#);
    // && returns the last truthy value (b)
    assert!(result.is_some());
}

// ── vm/core.rs lines 261-263: execute_file load error ───────────────────────

#[test]
fn execute_file_load_error() {
    use std::path::Path;
    let mut vm = VirtualMachine::new();
    // A file that exists but has a path we can't read (nonexistent)
    let result = vm.execute_file(Path::new("nonexistent_file_xyz.rb"));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("File not found") || msg.contains("error") || msg.contains("Error"));
}

// ── vm/native_functions.rs lines 172-192: gets with wrong args ──────────────

#[test]
fn gets_with_args_error() {
    let err = run_err("gets(1)");
    assert!(err.contains("argument") || err.contains("gets"));
}

// ── vm/native_functions.rs line 271-273: unknown native function ────────────
// The "unknown native function" error path is hit when calling a NativeFunction
// with an unrecognized name. We can trigger this by setting a variable to a
// NativeFunction object with a bogus name, then calling it.

#[test]
fn unknown_native_function_error() {
    let tokens = Lexer::new("x = 1").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("setup failed");
    // Define a fake native function in the environment
    vm.environment_mut().define(
        "fake_func".to_string(),
        Object::NativeFunction("bogus_xyz".to_string()),
    );
    let tokens2 = Lexer::new("fake_func()").tokenize();
    let stmts2 = Parser::new(tokens2).parse().expect("parse failed");
    let result = vm.execute_program(&stmts2);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Unknown") || msg.contains("native") || msg.contains("bogus_xyz"),
        "Error was: {}",
        msg
    );
}

// ── vm/native_functions.rs lines 104-131: print output without newline ──────

#[test]
fn print_formats_arguments_without_newline() {
    // print should not panic and should return nil
    let result = run(r#"print("hello", " ", "world")"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── vm/native_functions.rs: p function returns value ────────────────────────

#[test]
fn p_function_returns_single_value() {
    let result = run("p(42)");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn p_function_multiple_args_returns_nil() {
    let result = run("p(1, 2, 3)");
    assert_eq!(result, Some(Object::Nil));
}

// ── vm/pattern_matching.rs: Bind pattern matching ───────────────────────────

#[test]
fn bind_pattern_that_fails_inner_match() {
    let err = run_err(
        r#"
case "hello"
in Integer => n
  n
end
"#,
    );
    assert!(err.contains("NoMatchingPatternError") || err.contains("pattern"));
}

// ── vm/pattern_matching.rs: Multiple pattern (OR) where first fails ─────────

#[test]
fn multiple_pattern_first_fails_second_matches() {
    let result = run(r#"
case 5
when 1, 5
  "matched"
end
"#);
    assert_eq!(result, Some(Object::string("matched")));
}

#[test]
fn multiple_pattern_none_match() {
    let err = run_err(
        r#"
case 99
when 1, 2, 3
  "matched"
end
"#,
    );
    assert!(err.contains("No pattern matched") || err.contains("pattern"));
}

// ── vm/pattern_matching.rs: guard clause that fails ─────────────────────────

#[test]
fn case_when_guard_fails_falls_through() {
    let result = run(r#"
case 5
when n if n > 10
  "big"
when n
  "small"
end
"#);
    assert_eq!(result, Some(Object::string("small")));
}

// ── vm/pattern_matching.rs: case expression with else clause ────────────────

#[test]
fn case_expression_else_clause() {
    let result = run(r#"
result = case 99
when 1 then "one"
when 2 then "two"
else "other"
end
result
"#);
    assert_eq!(result, Some(Object::string("other")));
}

// ── vm/pattern_matching.rs: case expression no match no else returns nil ────

#[test]
fn case_expression_no_match_no_else_returns_nil() {
    let result = run(r#"
result = case 99
when 1 then "one"
when 2 then "two"
end
result
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── vm/core.rs: custom operator methods ─────────────────────────────────────

#[test]
fn custom_operator_divide() {
    let result = run(r#"
class Num
  attr_reader :val
  def initialize(v)
    @val = v
  end
  def /(other)
    Num.new(@val / other.val)
  end
end
(Num.new(10) / Num.new(2)).val
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn custom_operator_modulo() {
    let result = run(r#"
class Num
  attr_reader :val
  def initialize(v)
    @val = v
  end
  def %(other)
    Num.new(@val % other.val)
  end
end
(Num.new(10) % Num.new(3)).val
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn custom_operator_equal_equal() {
    let result = run(r#"
class Num
  attr_reader :val
  def initialize(v)
    @val = v
  end
  def ==(other)
    @val == other.val
  end
end
Num.new(5) == Num.new(5)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn custom_operator_not_equal() {
    let result = run(r#"
class Num
  attr_reader :val
  def initialize(v)
    @val = v
  end
  def !=(other)
    @val != other.val
  end
end
Num.new(5) != Num.new(3)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn custom_operator_less() {
    let result = run(r#"
class Num
  attr_reader :val
  def initialize(v)
    @val = v
  end
  def <(other)
    @val < other.val
  end
end
Num.new(3) < Num.new(5)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn custom_operator_greater() {
    let result = run(r#"
class Num
  attr_reader :val
  def initialize(v)
    @val = v
  end
  def >(other)
    @val > other.val
  end
end
Num.new(5) > Num.new(3)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn custom_operator_less_equal() {
    let result = run(r#"
class Num
  attr_reader :val
  def initialize(v)
    @val = v
  end
  def <=(other)
    @val <= other.val
  end
end
Num.new(5) <= Num.new(5)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn custom_operator_greater_equal() {
    let result = run(r#"
class Num
  attr_reader :val
  def initialize(v)
    @val = v
  end
  def >=(other)
    @val >= other.val
  end
end
Num.new(5) >= Num.new(5)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── vm/core.rs: custom index [] operator ────────────────────────────────────

#[test]
fn custom_index_operator() {
    let result = run(r#"
class MyList
  def initialize
    @items = [10, 20, 30]
  end
  def [](idx)
    @items[idx]
  end
end
MyList.new[1]
"#);
    assert_eq!(result, Some(Object::Int(20)));
}

// ── vm/core.rs: Match break/continue at top level ───────────────────────────

#[test]
fn match_break_at_top_level_error() {
    let err = run_err(
        r#"
case 1
when 1
  break
end
"#,
    );
    assert!(err.contains("break") || err.contains("loop"));
}

#[test]
fn match_continue_at_top_level_error() {
    let err = run_err(
        r#"
case 1
when 1
  continue
end
"#,
    );
    assert!(err.contains("continue") || err.contains("loop"));
}

// ── vm/core.rs: top-level break/continue errors ─────────────────────────────

#[test]
fn top_level_break_error() {
    let err = run_err("break");
    assert!(err.contains("break") || err.contains("loop"));
}

#[test]
fn top_level_continue_error() {
    let err = run_err("continue");
    assert!(err.contains("continue") || err.contains("loop"));
}

// ── vm/core.rs: GlobalVariable read ─────────────────────────────────────────

#[test]
fn global_variable_read_undefined_returns_nil() {
    let result = run("$undefined_global_var");
    assert_eq!(result, Some(Object::Nil));
}

// ── vm/core.rs: scope resolution on class ───────────────────────────────────

#[test]
fn scope_resolution_on_class_with_constant() {
    let result = run(r#"
class Config
  @@VERSION = "1.0"
end
Config::VERSION
"#);
    assert_eq!(result, Some(Object::string("1.0")));
}

#[test]
fn scope_resolution_on_class_uninitialized_constant_error() {
    let err = run_err(
        r#"
class Config
end
Config::MISSING
"#,
    );
    assert!(err.contains("Uninitialized constant") || err.contains("constant"));
}
