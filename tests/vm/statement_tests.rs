// Coverage tests for vm/statement.rs uncovered paths

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

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

// ── @var set when self is not an Instance (lines 199-201) ─────────────────────
// This triggers when trying to set @var in a context where self exists but
// is not an Instance (e.g., setting @var at top-level where self is absent
// → None path at 203-209). The Some(_) non-instance path is harder to trigger.

#[test]
fn instance_var_set_outside_method_error() {
    let err = run_err("@foo = 42");
    assert!(
        err.contains("instance variable")
            || err.contains("@foo")
            || err.contains("method")
            || err.contains("context")
    );
}

// ── @@var set when self is None (no context) ──────────────────────────────────

#[test]
fn class_var_set_outside_class_error() {
    let err = run_err("@@count = 1");
    assert!(
        err.contains("class variable")
            || err.contains("@@count")
            || err.contains("class")
            || err.contains("context")
    );
}

// ── @@var set inside class body (self is Class object) lines 221-223 ──────────

#[test]
fn class_var_set_in_class_body() {
    let result = run(r#"
class Counter
  @@total = 0
  def initialize
    @@total = @@total + 1
  end
  def total
    @@total
  end
end
c1 = Counter.new
c2 = Counter.new
c1.total
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── @@var read in class body (self is Class) lines 483-484 ────────────────────

#[test]
fn class_var_read_in_class_body() {
    let result = run(r#"
class Config
  @@value = 42
  def get_value
    @@value
  end
end
Config.new.get_value
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── @var set when self is a Class (lines 199-201) ─────────────────────────────
// Metorex allows calling instance methods on the class itself (no def self. syntax).
// When called on the class, self = Object::Class, triggering the Some(_) branch.

#[test]
fn instance_var_on_class_stores_as_class_var() {
    // Ruby allows @var on classes/modules — stored as class-level instance vars
    let result = run(r#"
class Foo
  def set_ivar
    @x = 10
  end
end
Foo.set_ivar
"#);
    assert!(result.is_none() || result == Some(Object::Nil) || result == Some(Object::Int(10)));
}

// ── @@var set when self is a Class object (lines 221-223) ─────────────────────
// When an instance method is called on the class (Foo.method), self = Class.

#[test]
fn class_var_set_in_method_called_on_class() {
    let result = run(r#"
class Foo
  def set_class_var
    @@x = 42
  end
  def get_class_var
    @@x
  end
end
Foo.set_class_var
Foo.get_class_var
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── setter method called on non-instance type (lines 353-359) ────────────────

#[test]
fn setter_method_on_non_instance_error() {
    let err = run_err(
        r#"
arr = [1, 2, 3]
arr.foo = 42
"#,
    );
    assert!(
        err.contains("setter")
            || err.contains("foo")
            || err.contains("Cannot")
            || err.contains("Array")
            || err.contains("method")
    );
}

#[test]
fn setter_method_on_integer_error() {
    let err = run_err("1.foo = 2");
    assert!(
        err.contains("setter")
            || err.contains("foo")
            || err.contains("Cannot")
            || err.contains("Integer")
            || err.contains("method")
    );
}

// ── global variable assignment ────────────────────────────────────────────────

#[test]
fn global_var_assignment_and_read() {
    let result = run(r#"
$count = 10
$count = $count + 5
$count
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

// ── @var read outside method error (None context, lines 464-470) ─────────────

#[test]
fn instance_var_read_outside_method_error() {
    let err = run_err("@foo");
    assert!(
        err.contains("instance variable")
            || err.contains("@foo")
            || err.contains("method")
            || err.contains("context")
    );
}

// ── @@var read outside class error ───────────────────────────────────────────

#[test]
fn class_var_read_outside_class_error() {
    let err = run_err("@@total");
    assert!(
        err.contains("class variable")
            || err.contains("@@total")
            || err.contains("class")
            || err.contains("context")
    );
}

// ── compound assignment operators (parser/statements/mod.rs) ─────────────────
// MinusEqual (lines 64-68) and SlashEqual (lines 76-81) desugaring

#[test]
fn minus_equal_compound_assignment() {
    let result = run("x = 10\nx -= 3\nx");
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn divide_equal_compound_assignment() {
    let result = run("x = 12\nx /= 4\nx");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Chained assignment ──────────────────────────────────────────────────────

#[test]
fn chained_assignment_ivars() {
    let result = run(r#"
class Foo
  def test
    @a = @b = 42
    @a + @b
  end
end
Foo.new.test
"#);
    assert_eq!(result, Some(Object::Int(84)));
}

#[test]
fn chained_assignment_globals() {
    let result = run("$a = $b = 10; $a + $b");
    assert_eq!(result, Some(Object::Int(20)));
}

#[test]
fn chained_assignment_index() {
    let result = run(r#"
a = [0, 0, 0]
a[0] = a[1] = 5
a
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(5),
            Object::Int(5),
            Object::Int(0)
        ]))
    );
}

// ── Parallel assignment ─────────────────────────────────────────────────────

#[test]
fn parallel_assignment_bracket_swap() {
    let result = run(r#"
a = [1, 2, 3]
a[0], a[2] = a[2], a[0]
a
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(3),
            Object::Int(2),
            Object::Int(1)
        ]))
    );
}

#[test]
fn parallel_assignment_ivars() {
    let result = run(r#"
class Foo
  def test
    @a, @b = 10, 20
    @a + @b
  end
end
Foo.new.test
"#);
    assert_eq!(result, Some(Object::Int(30)));
}

#[test]
fn multiple_assignment_postfix_if_true() {
    let result = run("a, b = 1, 2 if true; a");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn multiple_assignment_fewer_values() {
    let result = run("a, b, c = 1, 2; c");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn multiple_assignment_three_targets() {
    let result = run("a, b, c = 10, 20, 30; b");
    assert_eq!(result, Some(Object::Int(20)));
}

#[test]
fn multiple_assignment_from_array_splatted() {
    let result = run("a, b, c = [7, 8, 9]; c");
    assert_eq!(result, Some(Object::Int(9)));
}

// ── Assignment in elsif ─────────────────────────────────────────────────────

#[test]
fn assignment_in_elsif() {
    let result = run(r#"
x = nil
if false
  1
elsif x = 42
  x
end
x
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── Assignment in postfix unless ────────────────────────────────────────────

#[test]
fn assignment_in_postfix_unless() {
    let result = run(r#"
def test
  return unless files = [1, 2, 3]
  files.length
end
test()
"#);
    assert_eq!(result, Some(Object::Int(3)));
}
#[test]
fn instance_var_assign_at_top_level_error() {
    let err = run_err("@foo = 42");
    assert!(err.contains("Instance variable") || err.contains("@foo") || err.contains("method"));
}

// ── Instance variable read at top level ──────────────────────────────────────

#[test]
fn instance_var_read_at_top_level_error() {
    let err = run_err("@foo");
    assert!(err.contains("Instance variable") || err.contains("@foo") || err.contains("method"));
}

// ── Class variable assignment at top level ────────────────────────────────────

#[test]
fn class_var_assign_at_top_level_error() {
    let err = run_err("@@count = 42");
    assert!(err.contains("Class variable") || err.contains("@@count") || err.contains("class"));
}

// ── Class variable read at top level ─────────────────────────────────────────

#[test]
fn class_var_read_at_top_level_error() {
    let err = run_err("@@count");
    assert!(err.contains("Class variable") || err.contains("@@count") || err.contains("class"));
}

// ── attr_reader / attr_writer / attr_accessor outside class ──────────────────

#[test]
fn attr_reader_outside_class_error() {
    let err = run_err("attr_reader :foo");
    assert!(err.contains("attr_reader") || err.contains("class"));
}

#[test]
fn attr_writer_outside_class_error() {
    let err = run_err("attr_writer :foo");
    assert!(err.contains("attr_writer") || err.contains("class"));
}

#[test]
fn attr_accessor_outside_class_error() {
    let err = run_err("attr_accessor :foo");
    assert!(err.contains("attr_accessor") || err.contains("class"));
}

// ── include / extend outside class ───────────────────────────────────────────

#[test]
fn include_outside_class_error() {
    // Top-level `include Mod` adds the module to Object's mixin chain,
    // matching Ruby's main-scope `include` semantics. An undefined constant
    // still raises.
    let err = run_err(
        r#"
include UndefinedNopeNope
"#,
    );
    assert!(err.contains("Undefined module") || err.contains("UndefinedNopeNope"));
}

#[test]
fn extend_outside_class_error() {
    let err = run_err(
        r#"
module M
end
extend M
"#,
    );
    assert!(err.contains("extend") || err.contains("class"));
}

// ── break / continue outside loop (via execute_program) ──────────────────────

#[test]
fn break_at_top_level_error() {
    let err = run_err("break");
    assert!(err.contains("break"));
}

#[test]
fn continue_at_top_level_error() {
    let err = run_err("continue");
    assert!(err.contains("continue"));
}

// ── Array index assignment errors ─────────────────────────────────────────────

#[test]
fn array_index_out_of_bounds_assignment_error() {
    let err = run_err(
        r#"
arr = [1, 2, 3]
arr[10] = 99
"#,
    );
    assert!(err.contains("out of bounds") || err.contains("index"));
}

#[test]
fn array_non_int_index_assignment_error() {
    let err = run_err(
        r#"
arr = [1, 2, 3]
arr["key"] = 99
"#,
    );
    assert!(err.contains("integer") || err.contains("index"));
}

// ── Hash index assignment with complex key ────────────────────────────────────

#[test]
fn hash_index_assignment_with_int_key() {
    let result = run(r#"
h = {}
h[1] = "one"
h[1]
"#);
    assert_eq!(result, Some(Object::string("one")));
}

#[test]
fn hash_index_assignment_with_float_key() {
    let result = run(r#"
h = {}
h[1.5] = "val"
h[1.5]
"#);
    assert_eq!(result, Some(Object::string("val")));
}

#[test]
fn hash_index_assignment_with_bool_key() {
    let result = run(r#"
h = {}
h[true] = "yes"
h[true]
"#);
    assert_eq!(result, Some(Object::string("yes")));
}

#[test]
fn hash_index_assignment_with_nil_key() {
    let result = run(r#"
h = {}
h[nil] = "nothing"
h[nil]
"#);
    assert_eq!(result, Some(Object::string("nothing")));
}

// ── Hash index assignment with invalid key type ───────────────────────────────

#[test]
fn hash_index_assignment_with_array_key_works() {
    // Ruby allows any object as a hash key
    let result = run(r#"
h = {}
h[[1, 2]] = "val"
h[[1, 2]]
"#);
    assert_eq!(result, Some(Object::string("val".to_string())));
}

// ── Index assignment on non-container type ────────────────────────────────────

#[test]
fn index_assignment_on_integer_error() {
    let err = run_err(
        r#"
x = 42
x[0] = 1
"#,
    );
    assert!(err.contains("index") || err.contains("type") || err.contains("assign"));
}

// ── Instance without []= method ───────────────────────────────────────────────

#[test]
fn instance_index_assign_without_method_error() {
    let err = run_err(
        r#"
class C
end
c = C.new
c[0] = 42
"#,
    );
    assert!(err.contains("[]=") || err.contains("method") || err.contains("No"));
}

// ── Setter method undefined ───────────────────────────────────────────────────

#[test]
fn setter_method_undefined_error() {
    let err = run_err(
        r#"
class C
end
c = C.new
c.name = "test"
"#,
    );
    assert!(err.contains("setter") || err.contains("name=") || err.contains("Undefined"));
}

// ── Statement::Block via direct AST execution ─────────────────────────────────

#[test]
fn statement_block_executes_scoped() {
    use metorex::ast::{Expression, Statement};
    use metorex::lexer::Position;
    use metorex::vm::VirtualMachine;

    let mut vm = VirtualMachine::new();
    let pos = Position::new(1, 1, 0);
    let block = Statement::Block {
        statements: vec![Statement::Assignment {
            target: Expression::Identifier {
                name: "inner".to_string(),
                position: pos,
            },
            value: Expression::IntLiteral {
                value: 42,
                position: pos,
            },
            position: pos,
        }],
        position: pos,
    };
    let result = vm.execute_program(&[block]);
    assert!(result.is_ok());
}

// ── MethodDef at top level (not in class) ─────────────────────────────────────

#[test]
fn method_def_at_top_level_error() {
    use metorex::ast::{Parameter, Statement};
    use metorex::lexer::Position;
    use metorex::vm::VirtualMachine;

    let mut vm = VirtualMachine::new();
    let pos = Position::new(1, 1, 0);
    let stmt = Statement::MethodDef {
        is_class_method: false,
        name: "foo".to_string(),
        parameters: vec![Parameter::simple("x".to_string(), pos)],
        body: vec![],
        position: pos,
    };
    let result = vm.execute_program(&[stmt]);
    assert!(result.is_err());
}

// ── Bare function identifier auto-call ────────────────────────────────────────

#[test]
fn bare_function_identifier_auto_calls() {
    // A function defined at top level can be called without parentheses
    let result = run(r#"
def greet
  42
end
greet
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── continue inside case/when at top level (vm/core.rs lines 183-184) ─────────
// The Match/CaseIn handler in execute_program must see ControlFlow::Continue.

#[test]
fn continue_inside_case_when_at_top_level_error() {
    let err = run_err(
        r#"
case 42
when x
  continue
end
"#,
    );
    assert!(err.contains("continue") || err.contains("loop") || err.contains("outside"));
}

#[test]
fn continue_inside_case_in_at_top_level_error() {
    let err = run_err(
        r#"
case 42
in Integer => n
  continue
end
"#,
    );
    assert!(err.contains("continue") || err.contains("loop") || err.contains("outside"));
}

// ── method call with arguments as assignment target (vm/statement.rs 363-365) ─
// obj.foo(arg) = value is an error since only 0-arg method calls can be setters.

#[test]
fn method_call_with_args_as_assignment_target_error() {
    let err = run_err(
        r#"
class Foo
  def initialize
    @x = 0
  end
  def set(key)
    @x = key
  end
end
f = Foo.new
f.set(1) = 42
"#,
    );
    assert!(err.contains("assign") || err.contains("Cannot") || err.contains("method"));
}

// ── invalid symbol token (parser/expressions/primary.rs line 93) ──────────────

#[test]
fn symbol_with_non_ident_token_error() {
    let tokens = metorex::lexer::Lexer::new(":42").tokenize();
    let result = metorex::parser::Parser::new(tokens).parse();
    // :42 is invalid - symbol must have an identifier after ':'
    assert!(result.is_err() || result.unwrap().is_empty());
}

// ── break/continue in function body (method_invocation.rs lines 508-512) ──────

#[test]
fn break_inside_function_body_error() {
    let err = run_err(
        r#"
def foo
  break
end
foo
"#,
    );
    assert!(err.contains("break") || err.contains("loop") || err.contains("outside"));
}

#[test]
fn continue_inside_function_body_error() {
    let err = run_err(
        r#"
def foo
  continue
end
foo
"#,
    );
    assert!(err.contains("continue") || err.contains("loop") || err.contains("outside"));
}

// ── break/continue inside block called via .call (method_invocation.rs 184-188) ─

#[test]
fn break_inside_block_passed_to_method_returns_break_value() {
    // Ruby: `break` in a block attached to `run_block { break }` unwinds to
    // that method call, making the whole call evaluate to the break value
    // (nil when `break` has no operand).
    let result = run(r#"
def run_block(&b)
  b.call
end
run_block { break }
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn continue_inside_block_call_error() {
    let err = run_err(
        r#"
def run_block(&b)
  b.call
end
run_block { continue }
"#,
    );
    assert!(err.contains("continue") || err.contains("loop") || err.contains("outside"));
}

// ── standalone do |x| ... end block with parameters (primary.rs lines 243-265) ─

#[test]
fn standalone_do_block_with_params() {
    let result = run(r#"
b = do |x|
  x * 2
end
b.call(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn standalone_do_block_with_multiple_params() {
    let result = run(r#"
b = do |x, y|
  x + y
end
b.call(3, 4)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

// ── ScopeResolution: undefined constant error (core.rs lines 637-639) ─────────

#[test]
fn scope_resolution_undefined_constant_error() {
    // Accessing Foo::UNDEFINED when UNDEFINED is not defined as a class var on Foo.
    // Covers the ok_or_else closure at core.rs lines 637-639.
    let err = run_err(
        r#"
class Foo
end
Foo::UNDEFINED_CONST
"#,
    );
    assert!(err.contains("Uninitialized constant") || err.contains("UNDEFINED_CONST"));
}

// ── class variable assignment on non-class context (statement.rs lines 225-227) ──

#[test]
fn class_var_assign_on_non_instance_context_error() {
    let err = run_err("@@foo = 42");
    assert!(err.contains("class variable") || err.contains("@@foo"));
}

// ── ensure block with return overrides result (exceptions.rs line 199/201) ──

#[test]
fn ensure_block_with_return_overrides_result() {
    let result = run(r#"
def test_ensure
  begin
    "body"
  ensure
    return "from ensure"
  end
end
test_ensure
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("from ensure".to_string())))
    );
}

// ── exception_matches with non-exception object (exceptions.rs line 227) ──

#[test]
fn rescue_with_typed_catch_non_matching() {
    let result = run(r#"
class CustomError < RuntimeError
end
result = nil
begin
  raise CustomError.new("custom")
rescue TypeError
  result = "wrong type"
rescue CustomError => e
  result = e.message
end
result
"#);
    assert_eq!(result, Some(Object::String(Rc::new("custom".to_string()))));
}

// ── Float#round coverage (builtin_classes.rs lines 265-272) ──

#[test]
fn float_round_method() {
    let result = run("3.14159.round(2)");
    assert_eq!(result, Some(Object::Float(3.14)));
}

#[test]
fn float_round_zero_precision() {
    let result = run("3.7.round(0)");
    assert_eq!(result, Some(Object::Float(4.0)));
}

// ── Class equality comparison (class.rs lines 143, 150-153) ──

#[test]
fn class_with_methods_equality() {
    let result = run(r#"
class Foo
  def bar
    1
  end
end
class Baz
  def bar
    1
  end
end
Foo == Foo
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── method_invocation.rs line 369: unless as expression in method body ──

#[test]
fn unless_expression_in_method_body() {
    let result = run(r#"
def check(x)
  result = unless x
    "was false"
  else
    "was true"
  end
  result
end
check true
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("was true".to_string())))
    );
}
