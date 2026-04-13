// VM tests for method invocation, block params, lambdas, splat args.

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

// ── arity checks ─────────────────────────────────────────────────────────────

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

// ── method return values ──────────────────────────────────────────────────────

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

// ── block parameters ─────────────────────────────────────────────────────────

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

#[test]
fn method_with_block_param_called_with_lambda_as_positional() {
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

#[test]
fn function_with_block_param_called_without_block() {
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

#[test]
fn class_method_block_param_is_nil_when_no_block_passed() {
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

// ── standalone function ───────────────────────────────────────────────────────

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

// ── lambda ────────────────────────────────────────────────────────────────────

#[test]
fn lambda_brace_block() {
    let result = run(r#"
f = lambda { |x| x * 2 }
f.call(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn stabby_lambda_expression() {
    let result = run(r#"
f = ->(x) { x + 1 }
f.call(4)
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn stabby_lambda_with_expression_body() {
    let result = run(r#"
f = ->(x) x + 1
f.call(4)
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── attr_writer ───────────────────────────────────────────────────────────────

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

// ── splat params ──────────────────────────────────────────────────────────────

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

// ── method begin/rescue as last statement ─────────────────────────────────────

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

// ── nested method calls ───────────────────────────────────────────────────────

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

// ── non-local return from block ───────────────────────────────────────────────

#[test]
fn non_local_return_from_block_in_method() {
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

// ── multi-arg bracket / parenless splat ───────────────────────────────────────

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

// ── &non_block positional arg ────────────────────────────────────────────────

#[test]
fn ampersand_with_non_block_passes_as_positional() {
    let result = run(r#"
def take_one(x)
  x
end
take_one(&42)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}
