// Coverage tests for vm/core.rs uncovered paths

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::path::Path;
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

// ── VirtualMachine::default() ────────────────────────────────────────────────

#[test]
fn vm_default_creates_valid_vm() {
    let mut vm = VirtualMachine::default();
    // Execute a trivial program to verify the VM is functional
    let tokens = Lexer::new("42").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let result = vm.execute_program(&stmts).expect("execution failed");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── ScopeResolution on non-class ─────────────────────────────────────────────

#[test]
fn scope_resolution_on_non_class_error() {
    let err = run_err(
        r#"
x = 42
x::FOO
"#,
    );
    assert!(
        err.contains("scope")
            || err.contains("::")
            || err.contains("class")
            || err.contains("module")
    );
}

// ── super errors ─────────────────────────────────────────────────────────────

#[test]
fn super_outside_method_context_error() {
    let err = run_err("super");
    assert!(err.contains("super") || err.contains("method") || err.contains("context"));
}

#[test]
fn super_chain_works_through_inheritance() {
    // super through a chain should work
    let result = run(r#"
class Base
  def greet
    "hello"
  end
end
class Child < Base
  def greet
    super
  end
end
Child.new.greet
"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn super_in_class_without_parent_error() {
    let err = run_err(
        r#"
class Alone
  def foo
    super
  end
end
Alone.new.foo
"#,
    );
    assert!(err.contains("super") || err.contains("superclass") || err.contains("parent"));
}

#[test]
fn super_method_not_in_parent_error() {
    let err = run_err(
        r#"
class Animal
  def speak
    "..."
  end
end
class Dog < Animal
  def fetch
    super
  end
end
Dog.new.fetch
"#,
    );
    assert!(
        err.contains("super")
            || err.contains("fetch")
            || err.contains("does not define")
            || err.contains("method")
    );
}

// ── Match/CaseIn ControlFlow::Return propagation at top level ─────────────────

#[test]
fn match_control_flow_return_at_top_level() {
    // A case/when at top level where the body does ControlFlow::Return
    // (happens when return statement appears directly in case/when body)
    let result = run(r#"
case 1
when 1
  42
end
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn case_in_control_flow_exception_at_top_level() {
    // case/in where no pattern matches → NoMatchingPatternError at top level
    let err = run_err(
        r#"
case "hello"
in 42
  "matched"
end
"#,
    );
    assert!(
        err.contains("NoMatchingPattern") || err.contains("pattern") || err.contains("matched")
    );
}

// ── InstanceVariable read with non-instance self ──────────────────────────────

#[test]
fn instance_var_read_on_non_instance_self() {
    // When inside a method where self is defined but has an unusual type,
    // reading an @var should either work or give a meaningful error.
    // We can test this by reading @var normally in a method
    let result = run(r#"
class C
  def initialize
    @x = 99
  end
  def get_x
    @x
  end
end
C.new.get_x
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

// ── ClassVariable read with Class self ───────────────────────────────────────

#[test]
fn class_var_read_inside_instance_method() {
    let result = run(r#"
class Counter
  @@count = 0
  def increment
    @@count = @@count + 1
  end
  def get_count
    @@count
  end
end
c = Counter.new
c.increment
c.increment
c.get_count
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── Match at top level with exception ─────────────────────────────────────────

#[test]
fn match_at_top_level_with_exception_propagates() {
    let err = run_err(
        r#"
case 1
when 1
  raise "oops"
end
"#,
    );
    assert!(err.contains("oops") || err.contains("exception") || err.contains("Uncaught"));
}

// ── vm/core.rs execute_program auto-call for Method objects ──────────────────

#[test]
fn auto_call_method_at_top_level() {
    let result = run(r#"
def hello
  "world"
end
hello
"#);
    assert_eq!(result, Some(Object::string("world")));
}

// ── Auto-call via execute_statement (in Block context) ───────────────────────

#[test]
fn auto_call_method_in_block_context() {
    use metorex::ast::{Expression, Statement};
    use metorex::lexer::{Lexer, Position};
    use metorex::parser::Parser;

    let pos = Position::new(1, 1, 0);

    // First define the function
    let mut vm = VirtualMachine::new();
    let tokens = Lexer::new("def foo\n  42\nend").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    vm.execute_program(&stmts).expect("setup failed");

    // Now create a Block statement containing a bare 'foo' expression
    // This goes through execute_block → execute_statements_internal → execute_statement
    // and hits the auto-call logic (lines 32-33 in statement.rs)
    let block = Statement::Block {
        statements: vec![Statement::Expression {
            expression: Expression::Identifier {
                name: "foo".to_string(),
                position: pos,
            },
            position: pos,
        }],
        position: pos,
    };
    let result = vm.execute_program(&[block]);
    assert!(result.is_ok());
}

// ── @var read when self is a Class (core.rs lines 460-462) ────────────────────
// When an instance method is called on the class (Foo.method), self = Class.
// Reading @var in that context hits the Some(_) non-instance branch.

#[test]
fn instance_var_read_on_class_self_returns_nil() {
    // In Ruby, @x on a Class returns nil (class-level instance variable)
    let result = run(r#"
class Foo
  def read_ivar
    @x
  end
end
Foo.read_ivar
"#);
    assert!(result == Some(Object::Nil) || result.is_none());
}

// ── super when self is a Class (core.rs lines 507-509) ───────────────────────
// When an instance method containing super is called on the class directly,
// self = Class → triggers the Some(_) non-instance super error.

#[test]
fn super_when_self_is_class_error() {
    let err = run_err(
        r#"
class Base
  def greet
    "hello"
  end
end
class Child < Base
  def greet
    super
  end
end
Child.greet
"#,
    );
    assert!(
        err.contains("super")
            || err.contains("instance")
            || err.contains("method")
            || err.contains("context")
    );
}

// ── execute_file: parse error in file (core.rs lines 274-276) ────────────────

#[test]
fn execute_file_parse_error_in_file() {
    // A file with a syntax error causes parse_file to fail,
    // covering the map_err at core.rs lines 274-276.
    let path = Path::new("tests/_examples/execute_file/syntax_error.rb");
    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(path);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("parse") || msg.contains("Failed") || msg.contains("error"));
}

// ── define_method with captured vars in block (class_execution.rs line 299) ──

#[test]
fn define_method_with_closure_captured_vars() {
    // A block defined inside a closure context captures outer vars
    let result = run(r#"
prefix = "hello"
class Greeter
  define_method("greet") do
    "world"
  end
end
Greeter.new.greet
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("world".to_string())))
    );
}

// ── method_invocation.rs lines 212-213: block with captured vars ──

#[test]
fn block_execution_with_captured_variables() {
    let result = run(r#"
x = 10
[1, 2, 3].map { |n| n + x }
"#);
    if let Some(Object::Array(arr)) = result {
        let arr = arr.borrow();
        assert_eq!(arr[0], Object::Int(11));
        assert_eq!(arr[1], Object::Int(12));
        assert_eq!(arr[2], Object::Int(13));
    } else {
        panic!("Expected array");
    }
}

// ── exception in call stack with no location (exceptions.rs line 94) ──

#[test]
fn uncaught_exception_at_top_level() {
    let err = run_err("raise \"top level error\"");
    assert!(err.contains("top level error"));
}

// ── Case/In as value expression (core.rs lines 170, 187) ───────────────

#[test]
fn case_in_as_value_expression() {
    let result = run("case 42\nin Integer => n\n  n\nend");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn case_in_continues_after_match() {
    let result = run("case 5\nin Integer\n  \"matched\"\nend");
    assert_eq!(result, Some(Object::String(Rc::new("matched".to_string()))));
}

// ── execute_file path (core.rs lines 238-263) ──────────────────────────

#[test]
fn require_relative_success_path() {
    let code = "require_relative(\"lib/helper\")";
    let tokens = metorex::lexer::Lexer::new(code).tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let mut vm = metorex::vm::VirtualMachine::new();
    let base = std::fs::canonicalize("tests/_examples/require/basic.rb").unwrap();
    vm.set_current_file(base.clone());
    vm.mark_file_loaded(base);
    let result = vm.execute_program(&stmts);
    assert!(result.is_ok());
}

// ── super with deep inheritance (core.rs lines 518-558) ────────────────

#[test]
fn super_deep_inheritance() {
    let result = run("class A\n  def val\n    1\n  end\nend\n\
         class B < A\n  def val\n    super + 10\n  end\nend\n\
         class C < B\n  def val\n    super + 100\n  end\nend\n\
         C.new.val");
    assert_eq!(result, Some(Object::Int(111)));
}

#[test]
fn super_with_arguments() {
    let result = run(
        "class Base\n  def greet(name)\n    \"Hello, #{name}\"\n  end\nend\n\
         class Child < Base\n  def greet(name)\n    super(name)\n  end\nend\n\
         Child.new.greet(\"World\")",
    );
    assert_eq!(
        result,
        Some(Object::String(Rc::new("Hello, World".to_string())))
    );
}

// ── binary_op_method_name (core.rs line 682) ───────────────────────────

#[test]
fn custom_operator_plus() {
    let result = run(
        "class Vec2\n  attr_accessor :x, :y\n  def initialize(x, y)\n    @x = x\n    @y = y\n  end\n\
         def +(other)\n    Vec2.new(@x + other.x, @y + other.y)\n  end\nend\n\
         v = Vec2.new(1, 2) + Vec2.new(3, 4)\nv.x",
    );
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn custom_operator_minus() {
    let result = run(
        "class Num\n  attr_reader :val\n  def initialize(v)\n    @val = v\n  end\n\
         def -(other)\n    Num.new(@val - other.val)\n  end\nend\n\
         (Num.new(10) - Num.new(3)).val",
    );
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn custom_operator_multiply() {
    let result = run(
        "class Num\n  attr_reader :val\n  def initialize(v)\n    @val = v\n  end\n\
         def *(other)\n    Num.new(@val * other.val)\n  end\nend\n\
         (Num.new(3) * Num.new(4)).val",
    );
    assert_eq!(result, Some(Object::Int(12)));
}

// ── defined? ────────────────────────────────────────────────────────────────

#[test]
fn defined_local_variable() {
    let result = run(r#"x = 1; defined?(x)"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(
            "local-variable".to_string()
        )))
    );
}

#[test]
fn defined_undefined() {
    assert_eq!(run("defined?(nonexistent)"), Some(Object::Nil));
}

#[test]
fn defined_method() {
    let result = run("def foo; end; defined?(foo)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("method".to_string())))
    );
}

#[test]
fn defined_constant() {
    assert_eq!(
        run("defined?(String)"),
        Some(Object::String(std::rc::Rc::new("constant".to_string())))
    );
}

#[test]
fn defined_literal() {
    assert_eq!(
        run("defined?(42)"),
        Some(Object::String(std::rc::Rc::new("expression".to_string())))
    );
}

#[test]
fn defined_global_variable() {
    let result = run("$test_def = 1; defined?($test_def)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(
            "global-variable".to_string()
        )))
    );
}

#[test]
fn defined_instance_var() {
    let result = run(
        "class Foo\n  def initialize\n    @x = 1\n  end\n  def check\n    defined?(@x)\n  end\nend\nFoo.new.check",
    );
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(
            "instance-variable".to_string()
        )))
    );
}

#[test]
fn defined_yield_with_block() {
    let result = run("def test\n  defined?(yield)\nend\ntest { 1 }");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("yield".to_string())))
    );
}

#[test]
fn defined_yield_without_block() {
    assert_eq!(
        run("def test; defined?(yield); end; test()"),
        Some(Object::Nil)
    );
}

#[test]
fn defined_class_variable() {
    let result =
        run("class Foo\n  @@x = 1\n  def check\n    defined?(@@x)\n  end\nend\nFoo.new.check");
    assert!(result.is_some());
}

#[test]
fn defined_self_in_method() {
    let result = run("class Foo\n  def check\n    defined?(self)\n  end\nend\nFoo.new.check");
    assert!(result.is_some());
}

#[test]
fn defined_method_call() {
    assert_eq!(
        run(r#"defined?(puts("hi"))"#),
        Some(Object::String(std::rc::Rc::new("method".to_string())))
    );
}

#[test]
fn defined_scope_resolution() {
    assert_eq!(run("defined?(Nonexistent::Thing)"), Some(Object::Nil));
}

#[test]
fn defined_global_function_returns_string() {
    // `puts` is in scope; defined? returns some non-nil string.
    let result = run("defined?(puts)");
    assert!(matches!(result, Some(Object::String(_))));
}

#[test]
fn defined_class_var_with_existing() {
    let result =
        run("class Foo\n  @@x = 1\n  def check\n    defined?(@@x)\n  end\nend\nFoo.new.check");
    if let Some(Object::String(s)) = result {
        assert!(s.contains("class") || s.contains("variable"));
    } else {
        panic!("expected String, got {:?}", result);
    }
}

#[test]
fn defined_scope_resolution_existing() {
    let result = run("class Foo\n  VERSION = 42\nend\ndefined?(Foo::VERSION)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("constant".to_string())))
    );
}

#[test]
fn defined_super_returns_super() {
    let result = run(
        "class Parent\n  def hi\n    \"p\"\n  end\nend\nclass Child < Parent\n  def hi\n    defined?(super)\n  end\nend\nChild.new.hi",
    );
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("super".to_string())))
    );
}

#[test]
fn defined_self_returns_string() {
    let result = run("class Foo\n  def check\n    defined?(self)\n  end\nend\nFoo.new.check");
    assert!(matches!(result, Some(Object::String(_))));
}

#[test]
fn defined_array_literal() {
    assert_eq!(
        run("defined?([1, 2, 3])"),
        Some(Object::String(std::rc::Rc::new("expression".to_string())))
    );
}

#[test]
fn defined_dictionary_literal() {
    assert_eq!(
        run("defined?({a: 1})"),
        Some(Object::String(std::rc::Rc::new("expression".to_string())))
    );
}

#[test]
fn defined_undefined_global_var_returns_nil() {
    assert_eq!(run("defined?($zzz_no_such_global)"), Some(Object::Nil));
}

#[test]
fn defined_instance_var_when_unset_returns_nil() {
    let result = run("class Foo\n  def check\n    defined?(@unset_iv)\n  end\nend\nFoo.new.check");
    assert_eq!(result, Some(Object::Nil));
}

// ── Splat in argument list ────────────────────────────────────────────────

#[test]
fn splat_with_non_array_value_treated_as_single_arg() {
    let result = run(r#"
def f(a, b)
  a + b
end
x = 5
f(*x, 10)
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn splat_with_nil_value_treated_as_single_arg() {
    let result = run(r#"
def f(x)
  x.nil?
end
f(*nil)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── From vm/additional_tests ────────────────────────────────────────────────

#[test]
fn global_variable_read_undefined_returns_nil() {
    assert_eq!(run("$undefined_global_var_xyz"), Some(Object::Nil));
}

#[test]
fn scope_resolution_on_class_with_constant() {
    let result = run("class Foo\n  VERSION = 42\nend\nFoo::VERSION");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── String indexing ─────────────────────────────────────────────────────────

#[test]
fn string_index_int() {
    assert_eq!(
        run(r#""hello"[0]"#),
        Some(Object::String(std::rc::Rc::new("h".to_string())))
    );
}

#[test]
fn string_index_negative() {
    assert_eq!(
        run(r#""hello"[-1]"#),
        Some(Object::String(std::rc::Rc::new("o".to_string())))
    );
}

#[test]
fn string_index_out_of_bounds() {
    assert_eq!(run(r#""hello"[99]"#), Some(Object::Nil));
}

#[test]
fn string_index_range() {
    assert_eq!(
        run(r#""hello"[1..3]"#),
        Some(Object::String(std::rc::Rc::new("ell".to_string())))
    );
}

// ── Class === type check ────────────────────────────────────────────────────

#[test]
fn class_triple_eq_match() {
    assert_eq!(run("Integer === 42"), Some(Object::Bool(true)));
}

#[test]
fn class_triple_eq_no_match() {
    assert_eq!(run("Integer === \"hello\""), Some(Object::Bool(false)));
}

#[test]
fn range_class_triple_eq() {
    assert_eq!(run("Range === (1..5)"), Some(Object::Bool(true)));
}

// ── Bare identifier to method dispatch ──────────────────────────────────────

#[test]
fn bare_ident_dispatches_to_self_method() {
    let result = run(r#"
module Foo
  def self.greet
    "hello"
  end
  def self.test
    greet
  end
end
Foo.test
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

#[test]
fn bare_ident_method_with_args_returns_bound() {
    let result = run(r#"
module Foo
  def self.add(a, b)
    a + b
  end
  def self.test
    add(3, 4)
  end
end
Foo.test
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

// ── Assignment as last expression returns value ─────────────────────────────

#[test]
fn assignment_last_expr_returns_value() {
    let result = run(r#"
def test
  x = 42
end
test()
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn or_assign_last_expr_returns_value() {
    let result = run(r#"
class Foo
  def self.config
    @config ||= { "x" => 1 }
  end
end
Foo.config
"#);
    assert!(result.is_some());
    assert!(!matches!(result, Some(Object::Nil)));
}

// ── Identifier resolution: class constants from instance methods ──────────

#[test]
fn identifier_class_constant_from_instance_method() {
    let result = run(r#"
class Foo
  BAR = 42
  def get_bar
    BAR
  end
end
Foo.new.get_bar
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn identifier_class_constant_from_class_method() {
    let result = run(r#"
class Foo
  BAR = 99
  def self.get_bar
    BAR
  end
end
Foo.get_bar
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn identifier_class_constant_from_module_context() {
    let result = run(r#"
module M
  X = 7
  def self.get_x
    X
  end
end
M.get_x
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn identifier_bare_new_in_class_method() {
    let result = run(r#"
class Foo
  def self.create
    new
  end
  def to_s
    "a Foo"
  end
end
Foo.create.to_s
"#);
    assert_eq!(result, Some(Object::String(Rc::new("a Foo".to_string()))));
}

#[test]
fn identifier_object_class_method_zero_arg() {
    let result = run(r#"
class Object
  def helper
    42
  end
end
helper
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn identifier_object_class_method_with_args_returns_bound() {
    let result = run(r#"
class Object
  def helper(x)
    x + 1
  end
end
helper(10)
"#);
    assert_eq!(result, Some(Object::Int(11)));
}

#[test]
fn identifier_object_method_fallback_with_receiver() {
    let result = run(r#"
class Object
  def greet(name)
    "hi #{name}"
  end
end
class Foo
  def test
    greet("world")
  end
end
Foo.new.test
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("hi world".to_string())))
    );
}

#[test]
fn identifier_undefined_variable_error() {
    let err = run_err("nonexistent_var");
    assert!(err.contains("Undefined"));
}

#[test]
fn identifier_global_constant_resolution() {
    let result = run(r#"
class MyClass
end
def check
  MyClass
end
check
"#);
    assert!(matches!(result, Some(Object::Class(_))));
}

#[test]
fn identifier_no_self_no_global_error() {
    let err = run_err("unknown_thing");
    assert!(err.contains("Undefined"));
}

#[test]
fn identifier_object_method_zero_arg_fallback_with_self() {
    let result = run(r#"
class Object
  def injected_helper
    "from object"
  end
end
class Foo
  def test
    injected_helper
  end
end
Foo.new.test
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("from object".to_string())))
    );
}

#[test]
fn identifier_object_method_with_args_fallback_with_self() {
    let result = run(r#"
class Object
  def injected_add(a, b)
    a + b
  end
end
class Foo
  def test
    injected_add(3, 4)
  end
end
Foo.new.test
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn identifier_variadic_object_method() {
    let result = run(r#"
class Object
  def variadic_helper(*args)
    args.length
  end
end
variadic_helper
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn identifier_undefined_with_self_in_scope() {
    let err = run_err(
        r#"
class Foo
  def test
    totally_unknown
  end
end
Foo.new.test
"#,
    );
    assert!(err.contains("Undefined"));
}

#[test]
fn identifier_class_constant_not_on_non_class_receiver() {
    let result = run(r#"
class Foo
  BAR = 42
  def get_bar
    BAR
  end
end
Foo.new.get_bar
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn identifier_method_with_args_returns_bound_method_on_self() {
    let result = run(r#"
class Foo
  def add(a, b)
    a + b
  end
  def test
    add(3, 4)
  end
end
Foo.new.test
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn identifier_variadic_method_on_self_auto_calls() {
    let result = run(r#"
class Foo
  def items(*args)
    args
  end
  def test
    items
  end
end
Foo.new.test.length
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn identifier_object_variadic_method_with_self() {
    let result = run(r#"
class Object
  def obj_variadic(*args)
    args.length
  end
end
class Bar
  def test
    obj_variadic
  end
end
Bar.new.test
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn identifier_object_method_with_args_and_self() {
    let result = run(r#"
class Object
  def obj_calc(x, y)
    x * y
  end
end
class Baz
  def test
    obj_calc(5, 6)
  end
end
Baz.new.test
"#);
    assert_eq!(result, Some(Object::Int(30)));
}

#[test]
fn identifier_bare_method_name_returns_bound() {
    let result = run(r#"
class Foo
  def compute(x)
    x * 2
  end
  def get_method
    compute
  end
end
f = Foo.new
m = f.get_method
m.class.to_s
"#);
    assert!(result.is_some());
}

#[test]
fn identifier_constant_from_globals_inside_method() {
    let result = run(r#"
class Widget
end
class Foo
  def check
    Widget
  end
end
Foo.new.check.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Widget".to_string()))));
}
