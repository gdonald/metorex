// Coverage tests for vm/method_invocation.rs uncovered paths

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

// ── Function called with wrong arg count ──────────────────────────────────────

#[test]
fn function_too_few_args_error() {
    let err = run_err(
        r#"
def add(x, y)
  x + y
end
add 1
"#,
    );
    assert!(err.contains("argument") || err.contains("add") || err.contains("expected"));
}

#[test]
fn function_too_many_args_error() {
    let err = run_err(
        r#"
def greet(name)
  name
end
greet("Alice", "Bob")
"#,
    );
    assert!(err.contains("argument") || err.contains("greet") || err.contains("expected"));
}

// ── Exception.new with non-string argument ────────────────────────────────────

#[test]
fn exception_new_with_non_string_arg() {
    let result = run(r#"
e = RuntimeError.new(42)
e
"#);
    // Creates an exception object with a non-string arg (debug-formatted)
    assert!(result.is_some());
}

// ── Exception.new with more than one argument ─────────────────────────────────

#[test]
fn exception_new_with_too_many_args_error() {
    let err = run_err(
        r#"
RuntimeError.new("a", "b")
"#,
    );
    assert!(err.contains("argument") || err.contains("Exception") || err.contains("0 or 1"));
}

// ── Unless as last statement in standalone function body ──────────────────────

#[test]
fn function_body_ends_with_unless_else_taken() {
    let result = run(r#"
def check(x)
  unless x > 0
    "negative or zero"
  else
    "positive"
  end
end
check 5
"#);
    assert_eq!(result, Some(Object::string("positive")));
}

#[test]
fn function_body_ends_with_unless_then_taken() {
    let result = run(r#"
def check(x)
  unless x > 0
    "negative or zero"
  else
    "positive"
  end
end
check(-3)
"#);
    assert_eq!(result, Some(Object::string("negative or zero")));
}

// ── Unless as last statement in instance method body ─────────────────────────

#[test]
fn method_body_ends_with_unless_else_taken() {
    let result = run(r#"
class Checker
  def check(x)
    unless x > 0
      "non-positive"
    else
      "positive"
    end
  end
end
Checker.new.check(10)
"#);
    assert_eq!(result, Some(Object::string("positive")));
}

#[test]
fn method_body_ends_with_unless_then_taken() {
    let result = run(r#"
class Checker
  def check(x)
    unless x > 0
      "non-positive"
    else
      "positive"
    end
  end
end
Checker.new.check(-2)
"#);
    assert_eq!(result, Some(Object::string("non-positive")));
}

// ── Missing required keyword argument ─────────────────────────────────────────

#[test]
fn missing_required_keyword_arg_error() {
    let err = run_err(
        r#"
def greet(name:)
  name
end
greet()
"#,
    );
    assert!(
        err.contains("keyword")
            || err.contains("name")
            || err.contains("Missing")
            || err.contains("argument")
    );
}

// ── Break/Continue in method body (non-loop context) ─────────────────────────

#[test]
fn break_inside_method_body_error() {
    let err = run_err(
        r#"
class C
  def foo
    break
  end
end
C.new.foo
"#,
    );
    assert!(err.contains("break") || err.contains("loop") || err.contains("control"));
}

#[test]
fn continue_inside_method_body_error() {
    let err = run_err(
        r#"
class C
  def foo
    continue
  end
end
C.new.foo
"#,
    );
    assert!(err.contains("continue") || err.contains("loop") || err.contains("control"));
}

// ── Exception.new with zero args ──────────────────────────────────────────────

#[test]
fn exception_new_with_no_args() {
    let result = run(r#"
e = RuntimeError.new
e
"#);
    assert!(result.is_some());
}

// ── Block with captured vars (method_invocation.rs lines 212-213) ───────────

#[test]
fn block_captures_closure_vars() {
    let result = run("x = 10\nf = lambda do |y|\n  x + y\nend\nf.call(5)");
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn block_captures_multiple_vars() {
    let result = run("a = 1\nb = 2\nf = lambda do |c|\n  a + b + c\nend\nf.call(3)");
    assert_eq!(result, Some(Object::Int(6)));
}

// ── Splat/variadic ──────────────────────────────────────────────────────────

#[test]
fn splat_collect_args() {
    assert_eq!(
        run("def f(*args)\n  args.length\nend\nf(1, 2, 3)"),
        Some(Object::Int(3))
    );
}

#[test]
fn splat_expand_in_call() {
    assert_eq!(
        run("def add(a, b, c); a + b + c; end; add(*[10, 20, 30])"),
        Some(Object::Int(60))
    );
}

#[test]
fn splat_with_fixed_before() {
    let result =
        run("def log(level, *msgs)\n  msgs.length\nend\nlog(\"INFO\", \"a\", \"b\", \"c\")");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn splat_empty() {
    assert_eq!(
        run("def f(*args); args.length; end; f()"),
        Some(Object::Int(0))
    );
}

#[test]
fn splat_in_method_call() {
    assert_eq!(
        run("def sum(a, b); a + b; end; sum(*[3, 4])"),
        Some(Object::Int(7))
    );
}

#[test]
fn splat_in_dotted_call() {
    let result = run(
        "class Calc\n  def add(a, b, c)\n    a + b + c\n  end\nend\nargs = [1, 2, 3]\nCalc.new.add(*args)",
    );
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn class_method_variadic() {
    assert_eq!(
        run(
            "class Logger\n  def log(level, *msgs)\n    msgs.length\n  end\nend\nLogger.new.log(\"INFO\", \"a\", \"b\")"
        ),
        Some(Object::Int(2))
    );
}

#[test]
fn variadic_with_fixed_before_and_default() {
    let result = run("def test(a, *rest)\n  rest\nend\ntest(1, 2, 3, 4)");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(2),
            Object::Int(3),
            Object::Int(4)
        ]))
    );
}

// ── Yield ───────────────────────────────────────────────────────────────────

#[test]
fn yield_basic() {
    assert_eq!(
        run("def test\n  yield 10\nend\ntest { |x| x * 2 }"),
        Some(Object::Int(20))
    );
}

#[test]
fn yield_no_args() {
    assert_eq!(
        run("def test\n  yield\nend\ntest { 42 }"),
        Some(Object::Int(42))
    );
}

#[test]
fn yield_no_block_error() {
    let err = run_err("def f; yield; end; f()");
    assert!(err.contains("no block given"));
}

#[test]
fn yield_multiple_args() {
    assert_eq!(
        run("def test\n  yield 1, 2, 3\nend\ntest { |a, b, c| a + b + c }"),
        Some(Object::Int(6))
    );
}

#[test]
fn block_given_with_yield() {
    assert_eq!(
        run(
            "def m(x)\n  if block_given?\n    yield x\n  else\n    x * 2\n  end\nend\nm(5) { |n| n * 10 }"
        ),
        Some(Object::Int(50))
    );
}

#[test]
fn block_given_without_block() {
    assert_eq!(
        run("def m(x)\n  if block_given?\n    yield x\n  else\n    x * 2\n  end\nend\nm(5)"),
        Some(Object::Int(10))
    );
}

// ── Call fallback to self.method ────────────────────────────────────────────

#[test]
fn call_fallback_to_self_method() {
    let result = run(r#"
class Foo
  def greet(name)
    "hi " + name
  end
  def test
    greet("world")
  end
end
Foo.new.test
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hi world".to_string())))
    );
}

// ── Variadic insufficient args ─────────────────────────────────────────────

#[test]
fn variadic_method_with_required_before_splat_errors_on_too_few() {
    let err = run_err(
        r#"
def f(a, *rest)
  a
end
f
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn variadic_method_with_required_after_splat_errors_on_too_few() {
    let err = run_err(
        r#"
def f(*rest, last)
  last
end
f
"#,
    );
    assert!(err.contains("argument"));
}

// ── Method-level rescue propagating exception value out of body ───────────

#[test]
fn method_rescue_with_assignment_as_last_statement() {
    let result = run(r#"
def f
  begin
    x = 42
  rescue => e
    -1
  end
end
f
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn method_rescue_else_clause_value_propagates() {
    let result = run(r#"
def f
  begin
    1
  rescue => e
    -1
  else
    99
  end
end
f
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn method_with_begin_ensure_propagates_body_value() {
    let result = run(r#"
def f
  begin
    100
  ensure
    200
  end
end
f
"#);
    assert_eq!(result, Some(Object::Int(100)));
}

// ── &block trailing arg passed positionally on instance method ────────────

#[test]
fn ampersand_block_arg_extracted_to_pending_block() {
    let result = run(r#"
class C
  def calls(&blk)
    blk.call(7)
  end
end
square = lambda { |x| x * x }
C.new.calls(&square)
"#);
    assert_eq!(result, Some(Object::Int(49)));
}

// ── return from inside a Begin propagates to method ───────────────────────

#[test]
fn return_from_begin_block_returns_from_method() {
    let result = run(r#"
def f
  begin
    return 99
  rescue => e
    -1
  end
  -2
end
f
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn return_from_rescue_block_returns_from_method() {
    let result = run(r#"
def f
  begin
    raise "boom"
  rescue => e
    return 88
  end
  -2
end
f
"#);
    assert_eq!(result, Some(Object::Int(88)));
}
