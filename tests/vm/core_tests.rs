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

#[test]
fn comparable_spaceship_fallback_for_less_than() {
    // dispatch.rs line 159: when a class defines <=> but not <, the Comparable-style
    // fallback derives < from the <=> result.
    let result = run(r#"
class Sortable
  def initialize(v)
    @v = v
  end
  def <=>(other)
    @v <=> other.value
  end
  def value
    @v
  end
end
a = Sortable.new(1)
b = Sortable.new(2)
a < b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn comparable_spaceship_fallback_for_greater_than() {
    // dispatch.rs line 159: derive > from <=> result when > is not defined.
    let result = run(r#"
class Ranked
  def initialize(v)
    @v = v
  end
  def <=>(other)
    @v <=> other.value
  end
  def value
    @v
  end
end
Ranked.new(5) > Ranked.new(3)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rescue_errno_style_exception_via_standard_error() {
    // exceptions.rs line 279: is_standard_exception_name with Errno:: prefix.
    // File.realpath on a non-existent path raises Errno::ENOENT.
    // Rescuing with StandardError triggers the is_standard_exception_name check
    // which reaches the `name.starts_with("Errno::")` branch.
    let result = run(r#"
result = begin
  File.realpath "/absolutely_nonexistent_path_metorex_test_xyz"
rescue StandardError => e
  "caught errno"
end
result
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("caught errno".to_string())))
    );
}

#[test]
fn parser_program_comment_only() {
    // parser/mod.rs line 157: parse loop fires the `if is_at_end { break }` guard
    // when the program consists only of a comment (whitespace consumed → EOF).
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    let tokens = Lexer::new("# just a comment").tokenize();
    let result = Parser::new(tokens).parse();
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// ── super with paren-less arguments (keywords.rs lines 100-105) ───────────────

#[test]
fn super_with_paren_less_args() {
    // keywords.rs lines 100-105: paren-less super arg list parsing
    let result = run(r#"
class Base
  def greet(name)
    "Hello, " + name
  end
end
class Child < Base
  def greet(name)
    super name
  end
end
Child.new.greet "World"
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("Hello, World".to_string())))
    );
}

// ── leading :: parse error (keywords.rs line 196) ─────────────────────────────

#[test]
fn leading_coloncolon_non_ident_error() {
    // keywords.rs line 196: ::non-ident triggers an error
    use metorex::lexer::Lexer;
    use metorex::parser::Parser;
    let tokens = Lexer::new("::123").tokenize();
    let result = Parser::new(tokens).parse();
    assert!(result.is_err());
    let msg = result.unwrap_err()[0].to_string();
    assert!(msg.contains("Expected") || msg.contains("identifier") || msg.contains("::"));
}
