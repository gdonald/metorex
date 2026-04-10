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

fn parse_ok(code: &str) {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed");
}

// ── =~ and !~ regex match operators ─────────────────────────────────────────

#[test]
fn regex_match_returns_position() {
    let result = run(r#"/hello/ =~ "say hello world""#);
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn regex_match_returns_nil_on_no_match() {
    let result = run(r#"/xyz/ =~ "hello""#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn regex_match_string_left() {
    let result = run(r#""test123" =~ /\d+/"#);
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn regex_not_match_true() {
    let result = run(r#""abc" !~ /xyz/"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn regex_not_match_false() {
    let result = run(r#""abc" !~ /abc/"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn regex_match_case_insensitive() {
    let result = run(r#"/hello/i =~ "HELLO world""#);
    assert_eq!(result, Some(Object::Int(0)));
}

// ── XOR operator ────────────────────────────────────────────────────────────

#[test]
fn xor_bool_true_false() {
    assert_eq!(run("true ^ false"), Some(Object::Bool(true)));
}

#[test]
fn xor_bool_true_true() {
    assert_eq!(run("true ^ true"), Some(Object::Bool(false)));
}

#[test]
fn xor_bool_false_false() {
    assert_eq!(run("false ^ false"), Some(Object::Bool(false)));
}

#[test]
fn xor_int() {
    assert_eq!(run("5 ^ 3"), Some(Object::Int(6)));
}

#[test]
fn xor_bool_with_truthy() {
    assert_eq!(run("true ^ nil"), Some(Object::Bool(true)));
}

// ── ||= and &&= ────────────────────────────────────────────────────────────

#[test]
fn or_assign_nil() {
    assert_eq!(run("x = nil; x ||= 42; x"), Some(Object::Int(42)));
}

#[test]
fn or_assign_existing() {
    assert_eq!(run("x = 10; x ||= 42; x"), Some(Object::Int(10)));
}

#[test]
fn and_assign_truthy() {
    assert_eq!(run("x = true; x &&= 42; x"), Some(Object::Int(42)));
}

#[test]
fn and_assign_falsy() {
    assert_eq!(run("x = false; x &&= 42; x"), Some(Object::Bool(false)));
}

// ── === triple equals ───────────────────────────────────────────────────────

#[test]
fn triple_equals() {
    assert_eq!(run("1 === 1"), Some(Object::Bool(true)));
}

#[test]
fn triple_equals_false() {
    assert_eq!(run("1 === 2"), Some(Object::Bool(false)));
}

// ── Stabby lambda -> ────────────────────────────────────────────────────────

#[test]
fn stabby_lambda_brace() {
    assert_eq!(run("f = -> { 42 }; f.call"), Some(Object::Int(42)));
}

#[test]
fn stabby_lambda_with_params() {
    assert_eq!(
        run("f = -> { |x| x * 2 }; f.call(5)"),
        Some(Object::Int(10))
    );
}

#[test]
fn stabby_lambda_as_argument() {
    assert_eq!(
        run("def apply(fn); fn.call; end; apply(-> { 99 })"),
        Some(Object::Int(99))
    );
}

// ── Symbol patterns in case/when ────────────────────────────────────────────

#[test]
fn case_when_symbol() {
    let result = run(r#"
case :hello
when :hello
  42
when :world
  99
end
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn case_when_symbol_no_match() {
    let result = run(r#"
case :other
when :hello
  42
when :world
  99
else
  nil
end
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── Keyword symbols ─────────────────────────────────────────────────────────

#[test]
fn symbol_from_keyword_class() {
    assert_eq!(
        run(":class"),
        Some(Object::Symbol(std::rc::Rc::new("class".to_string())))
    );
}

#[test]
fn symbol_from_ivar() {
    assert_eq!(
        run(":@name"),
        Some(Object::Symbol(std::rc::Rc::new("@name".to_string())))
    );
}

#[test]
fn symbol_from_cvar() {
    assert_eq!(
        run(":@@count"),
        Some(Object::Symbol(std::rc::Rc::new("@@count".to_string())))
    );
}

#[test]
fn symbol_from_string_literal() {
    assert_eq!(
        run(r#":"hello""#),
        Some(Object::Symbol(std::rc::Rc::new("hello".to_string())))
    );
}

// ── Keyword names in attr_reader ────────────────────────────────────────────

#[test]
fn attr_reader_with_include_keyword() {
    parse_ok(
        r#"
class Foo
  class << self
    attr_reader :include, :exclude
  end
end
"#,
    );
}

// ── Setter method def ───────────────────────────────────────────────────────

#[test]
fn setter_method_def() {
    let result = run(r#"
class Foo
  def name=(val)
    @name = val
  end
  def name
    @name
  end
end
f = Foo.new
f.name = "hello"
f.name
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
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

// ── Parallel assignment with brackets ───────────────────────────────────────

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
            Object::Int(1),
        ]))
    );
}

// ── Multiple assignment with postfix if ─────────────────────────────────────

#[test]
fn multiple_assignment_postfix_if_true() {
    let result = run("a, b = 1, 2 if true; a");
    assert_eq!(result, Some(Object::Int(1)));
}

// ── return if / return unless ───────────────────────────────────────────────

#[test]
fn return_if_true() {
    assert_eq!(
        run("def f; return 42 if true; 0; end; f()"),
        Some(Object::Int(42))
    );
}

#[test]
fn return_if_false() {
    assert_eq!(
        run("def f; return 42 if false; 0; end; f()"),
        Some(Object::Int(0))
    );
}

#[test]
fn return_unless_true() {
    assert_eq!(
        run("def f; return 42 unless true; 0; end; f()"),
        Some(Object::Int(0))
    );
}

#[test]
fn return_unless_false() {
    assert_eq!(
        run("def f; return 42 unless false; 0; end; f()"),
        Some(Object::Int(42))
    );
}

// ── %r() regex literal ──────────────────────────────────────────────────────

#[test]
fn percent_r_regex() {
    let result = run(r#"%r(hello) =~ "say hello""#);
    assert_eq!(result, Some(Object::Int(4)));
}

// ── %[...] string literal ───────────────────────────────────────────────────

#[test]
fn percent_bracket_string() {
    let result = run(r#"%[hello world]"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello world".to_string())))
    );
}

// ── Module body with instance variables ─────────────────────────────────────

#[test]
fn module_body_ivar_assignment() {
    parse_ok(
        r#"
module MSpec
  @exit = nil
  @abort = nil
end
"#,
    );
}

#[test]
fn module_body_class_self_attr() {
    parse_ok(
        r#"
module Foo
  class << self
    attr_reader :bar
  end
end
"#,
    );
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
    let result = run("defined?(String)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("constant".to_string())))
    );
}

#[test]
fn defined_literal() {
    let result = run("defined?(42)");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("expression".to_string())))
    );
}

// ── yield ───────────────────────────────────────────────────────────────────

#[test]
fn yield_basic() {
    let result = run(r#"
def test
  yield 10
end
test { |x| x * 2 }
"#);
    assert_eq!(result, Some(Object::Int(20)));
}

#[test]
fn yield_no_args() {
    let result = run(r#"
def test
  yield
end
test { 42 }
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn yield_no_block_error() {
    let err = run_err("def f; yield; end; f()");
    assert!(err.contains("no block given"));
}

// ── Splat ───────────────────────────────────────────────────────────────────

#[test]
fn splat_collect_args() {
    let result = run(r#"
def f(*args)
  args.length
end
f(1, 2, 3)
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn splat_expand_in_call() {
    let result = run("def add(a, b, c); a + b + c; end; add(*[10, 20, 30])");
    assert_eq!(result, Some(Object::Int(60)));
}

// ── Lambda [] call ──────────────────────────────────────────────────────────

#[test]
fn lambda_bracket_call_single_arg() {
    assert_eq!(run("f = lambda { |x| x * 3 }; f[7]"), Some(Object::Int(21)));
}

#[test]
fn lambda_bracket_call_multi_arg() {
    assert_eq!(
        run("f = lambda { |a, b| a + b }; f[3, 4]"),
        Some(Object::Int(7))
    );
}

// ── Stabby lambda -> forms ──────────────────────────────────────────────────

#[test]
fn stabby_lambda_do_end() {
    assert_eq!(
        run(r#"
f = -> do
  42
end
f.call
"#),
        Some(Object::Int(42))
    );
}

#[test]
fn stabby_lambda_parens_brace() {
    assert_eq!(
        run("f = -> (x) { x + 1 }; f.call(9)"),
        Some(Object::Int(10))
    );
}

#[test]
fn stabby_lambda_parens_multi() {
    assert_eq!(
        run("f = -> (a, b) { a * b }; f.call(3, 4)"),
        Some(Object::Int(12))
    );
}

#[test]
fn stabby_lambda_no_body_expr() {
    // -> expr (no braces, no do)
    assert_eq!(run("f = -> 42; f.call"), Some(Object::Int(42)));
}

// ── Interpolated symbol :"@#{expr}" ─────────────────────────────────────────

#[test]
fn interpolated_symbol_dynamic() {
    // :"@#{expr}" returns a string (dynamic symbol)
    let result = run(r#"x = "name"; :"@#{x}""#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("@name".to_string())))
    );
}

// ── More keyword symbols ────────────────────────────────────────────────────

#[test]
fn symbol_def() {
    assert_eq!(
        run(":def"),
        Some(Object::Symbol(std::rc::Rc::new("def".to_string())))
    );
}

#[test]
fn symbol_if() {
    assert_eq!(
        run(":if"),
        Some(Object::Symbol(std::rc::Rc::new("if".to_string())))
    );
}

#[test]
fn symbol_else() {
    assert_eq!(
        run(":else"),
        Some(Object::Symbol(std::rc::Rc::new("else".to_string())))
    );
}

#[test]
fn symbol_end() {
    assert_eq!(
        run(":end"),
        Some(Object::Symbol(std::rc::Rc::new("end".to_string())))
    );
}

#[test]
fn symbol_do() {
    assert_eq!(
        run(":do"),
        Some(Object::Symbol(std::rc::Rc::new("do".to_string())))
    );
}

#[test]
fn symbol_nil() {
    assert_eq!(
        run(":nil"),
        Some(Object::Symbol(std::rc::Rc::new("nil".to_string())))
    );
}

#[test]
fn symbol_true() {
    assert_eq!(
        run(":true"),
        Some(Object::Symbol(std::rc::Rc::new("true".to_string())))
    );
}

#[test]
fn symbol_false() {
    assert_eq!(
        run(":false"),
        Some(Object::Symbol(std::rc::Rc::new("false".to_string())))
    );
}

#[test]
fn symbol_return() {
    assert_eq!(
        run(":return"),
        Some(Object::Symbol(std::rc::Rc::new("return".to_string())))
    );
}

#[test]
fn symbol_begin() {
    assert_eq!(
        run(":begin"),
        Some(Object::Symbol(std::rc::Rc::new("begin".to_string())))
    );
}

#[test]
fn symbol_rescue() {
    assert_eq!(
        run(":rescue"),
        Some(Object::Symbol(std::rc::Rc::new("rescue".to_string())))
    );
}

#[test]
fn symbol_ensure() {
    assert_eq!(
        run(":ensure"),
        Some(Object::Symbol(std::rc::Rc::new("ensure".to_string())))
    );
}

#[test]
fn symbol_while() {
    assert_eq!(
        run(":while"),
        Some(Object::Symbol(std::rc::Rc::new("while".to_string())))
    );
}

#[test]
fn symbol_for() {
    assert_eq!(
        run(":for"),
        Some(Object::Symbol(std::rc::Rc::new("for".to_string())))
    );
}

#[test]
fn symbol_case() {
    assert_eq!(
        run(":case"),
        Some(Object::Symbol(std::rc::Rc::new("case".to_string())))
    );
}

#[test]
fn symbol_when() {
    assert_eq!(
        run(":when"),
        Some(Object::Symbol(std::rc::Rc::new("when".to_string())))
    );
}

#[test]
fn symbol_module() {
    assert_eq!(
        run(":module"),
        Some(Object::Symbol(std::rc::Rc::new("module".to_string())))
    );
}

#[test]
fn symbol_include() {
    assert_eq!(
        run(":include"),
        Some(Object::Symbol(std::rc::Rc::new("include".to_string())))
    );
}

#[test]
fn symbol_yield() {
    assert_eq!(
        run(":yield"),
        Some(Object::Symbol(std::rc::Rc::new("yield".to_string())))
    );
}

#[test]
fn symbol_super() {
    assert_eq!(
        run(":super"),
        Some(Object::Symbol(std::rc::Rc::new("super".to_string())))
    );
}

#[test]
fn symbol_lambda() {
    assert_eq!(
        run(":lambda"),
        Some(Object::Symbol(std::rc::Rc::new("lambda".to_string())))
    );
}

#[test]
fn symbol_break() {
    assert_eq!(
        run(":break"),
        Some(Object::Symbol(std::rc::Rc::new("break".to_string())))
    );
}

#[test]
fn symbol_next() {
    assert_eq!(
        run(":next"),
        Some(Object::Symbol(std::rc::Rc::new("next".to_string())))
    );
}

#[test]
fn symbol_raise() {
    assert_eq!(
        run(":raise"),
        Some(Object::Symbol(std::rc::Rc::new("raise".to_string())))
    );
}

// ── Module body execution ───────────────────────────────────────────────────

#[test]
fn module_body_ivar_and_method() {
    let result = run(r#"
module Config
  @debug = true

  def self.debug
    @debug
  end
end
"#);
    // Module definition returns nil
    assert!(result.is_none() || result == Some(Object::Nil));
}

#[test]
fn module_class_self_method_def() {
    let result = run(r#"
module Helpers
  class << self
    def answer
      42
    end
  end
end
"#);
    assert!(result.is_none() || result == Some(Object::Nil));
}

// ── Keyword attr_reader in class ────────────────────────────────────────────

#[test]
fn attr_reader_keyword_names() {
    parse_ok(
        r#"
class Foo
  attr_reader :include
  attr_reader :extend
  attr_reader :class
  attr_reader :module
  attr_reader :def
  attr_reader :end
  attr_reader :if
  attr_reader :else
  attr_reader :do
end
"#,
    );
}

#[test]
fn attr_writer_keyword_names() {
    parse_ok(
        r#"
class Foo
  attr_writer :include, :extend
end
"#,
    );
}

#[test]
fn attr_accessor_keyword_names() {
    parse_ok(
        r#"
class Foo
  attr_accessor :include, :extend
end
"#,
    );
}

// ── Chained assignment variations ───────────────────────────────────────────

#[test]
fn chained_assignment_globals() {
    let result = run("$a = $b = 10; $a + $b");
    assert_eq!(result, Some(Object::Int(20)));
}

// ── Parallel assignment with ivars ──────────────────────────────────────────

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

// ── defined? additional ─────────────────────────────────────────────────────

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
    let result = run(r#"
class Foo
  def initialize
    @x = 1
  end
  def check
    defined?(@x)
  end
end
Foo.new.check
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(
            "instance-variable".to_string()
        )))
    );
}

#[test]
fn defined_yield_with_block() {
    let result = run(r#"
def test
  defined?(yield)
end
test { 1 }
"#);
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

// ── %r with different delimiters ────────────────────────────────────────────

#[test]
fn percent_r_brackets() {
    let result = run(r#"%r[hello] =~ "hello world""#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn percent_r_braces() {
    let result = run(r#"%r{test} =~ "a test""#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── %Q and %() strings ─────────────────────────────────────────────────────

#[test]
fn percent_q_string() {
    assert_eq!(
        run(r#"%Q[hello]"#),
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

#[test]
fn percent_paren_string() {
    assert_eq!(
        run(r#"%(hello world)"#),
        Some(Object::String(std::rc::Rc::new("hello world".to_string())))
    );
}

#[test]
fn percent_brace_string() {
    assert_eq!(
        run(r#"%{hello}"#),
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

#[test]
fn percent_angle_string() {
    assert_eq!(
        run(r#"%<hello>"#),
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

// ── -> inside parenthesized arguments (hits parse_primary Arrow) ────────────

#[test]
fn stabby_lambda_in_parens_arg_brace() {
    // -> { } inside parenthesized call — goes through parse_expression, not parse_expression_with_lambda
    assert_eq!(
        run("def run(fn); fn.call; end; run(-> { 77 })"),
        Some(Object::Int(77))
    );
}

#[test]
fn stabby_lambda_in_parens_arg_do() {
    assert_eq!(
        run(r#"
def run(fn)
  fn.call
end
run(-> do
  88
end)
"#),
        Some(Object::Int(88))
    );
}

#[test]
fn stabby_lambda_in_parens_arg_with_params() {
    assert_eq!(
        run("def run(fn); fn.call(5); end; run(-> (x) { x * 3 })"),
        Some(Object::Int(15))
    );
}

#[test]
fn stabby_lambda_in_parens_arg_expr() {
    assert_eq!(
        run("def run(fn); fn.call; end; run(-> 42)"),
        Some(Object::Int(42))
    );
}

// ── class_execution: module with class << self attr + methods ───────────────

#[test]
fn module_class_self_attr_writer() {
    run(r#"
module Conf
  class << self
    attr_writer :debug
  end
end
"#);
}

#[test]
fn module_class_self_attr_accessor() {
    run(r#"
module Conf
  class << self
    attr_accessor :verbose
  end
end
"#);
}

#[test]
fn module_class_self_method_and_attr() {
    run(r#"
module Helpers
  class << self
    attr_reader :count

    def reset
      42
    end
  end
end
"#);
}

// ── more defined? paths ─────────────────────────────────────────────────────

#[test]
fn defined_self_in_method() {
    // self is stored as a variable in the environment
    let result = run(r#"
class Foo
  def check
    defined?(self)
  end
end
Foo.new.check
"#);
    // self is found as a local variable in method scope
    assert!(result.is_some());
    if let Some(Object::String(s)) = &result {
        assert!(s.as_ref() == "self" || s.as_ref() == "local-variable");
    }
}

#[test]
fn defined_method_call() {
    let result = run(r#"defined?(puts("hi"))"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("method".to_string())))
    );
}

#[test]
fn defined_scope_resolution() {
    // MSpec::VERSION doesn't exist but the check returns nil
    assert_eq!(run("defined?(Nonexistent::Thing)"), Some(Object::Nil));
}

// ── Splat: variadic with fixed params before and after ──────────────────────

#[test]
fn splat_with_fixed_before() {
    let result = run(r#"
def log(level, *msgs)
  msgs.length
end
log("INFO", "a", "b", "c")
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn splat_empty() {
    let result = run("def f(*args); args.length; end; f()");
    assert_eq!(result, Some(Object::Int(0)));
}

// ── Chained ternary with method calls ───────────────────────────────────────

#[test]
fn chained_ternary_with_methods() {
    let result = run(r#"
a = "hello"
x = a.length > 10 ? "long" : a.length > 3 ? "medium" : "short"
x
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("medium".to_string())))
    );
}

// ── Assignment in elsif condition ───────────────────────────────────────────

#[test]
fn assignment_in_elsif() {
    // Test that assignment in elsif condition works (parses and executes)
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

// ── Multiple assignment: more targets than values ───────────────────────────

#[test]
fn multiple_assignment_fewer_values() {
    let result = run("a, b, c = 1, 2; c");
    assert_eq!(result, Some(Object::Nil));
}

// ── method-level rescue with ensure ─────────────────────────────────────────

#[test]
fn method_rescue_ensure() {
    // Method-level rescue + ensure (no explicit begin)
    parse_ok(
        r#"
def test
  raise "oops"
rescue
  42
ensure
  99
end
"#,
    );
}

// ── Method-level rescue with else clause ─────────────────────────────────────

#[test]
fn method_rescue_else() {
    parse_ok(
        r#"
def safe_op
  42
rescue => e
  -1
else
  99
end
"#,
    );
}

#[test]
fn method_rescue_else_ensure() {
    parse_ok(
        r#"
def full_method
  42
rescue => e
  -1
else
  99
ensure
  0
end
"#,
    );
}

// ── Singleton setter method: def self.name=(...) ────────────────────────────

#[test]
fn singleton_setter_method() {
    parse_ok(
        r#"
class Foo
  def self.name=(val)
    @name = val
  end
end
"#,
    );
}

// ── XOR edge cases ──────────────────────────────────────────────────────────

#[test]
fn xor_non_bool_left() {
    // "hello" ^ true — truthy string XOR true
    assert_eq!(run(r#""hello" ^ true"#), Some(Object::Bool(false)));
}

#[test]
fn xor_non_bool_left_false() {
    // nil ^ false — falsy nil XOR false
    assert_eq!(run("nil ^ false"), Some(Object::Bool(false)));
}

// ── Hash [] method ──────────────────────────────────────────────────────────

#[test]
fn hash_bracket_access() {
    let result = run(r#"
h = { "a" => 1, "b" => 2 }
h["a"]
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_bracket_missing_key() {
    let err = run_err(
        r#"
h = { "a" => 1 }
h["missing"]
"#,
    );
    assert!(err.contains("not found") || err.contains("Key"));
}

// ── Range methods each/map block error paths ────────────────────────────────

#[test]
fn range_each_basic() {
    let result = run(r#"
result = []
(1..3).each do |x|
  result << x
end
result
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
        ]))
    );
}

#[test]
fn range_map_basic() {
    let result = run(r#"
(1..3).map do |x|
  x * 2
end
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(2),
            Object::Int(4),
            Object::Int(6),
        ]))
    );
}

// ── Set methods ─────────────────────────────────────────────────────────────

#[test]
fn set_size() {
    let result = run(r#"
s = Set.new([1, 2, 3])
s.size
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Hash each ───────────────────────────────────────────────────────────────

#[test]
fn hash_each_basic() {
    let result = run(r#"
h = { "a" => 1, "b" => 2 }
total = 0
h.each do |k, v|
  total = total + v
end
total
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Spaceship operator string comparison ────────────────────────────────────

#[test]
fn spaceship_string() {
    assert_eq!(run(r#""abc" <=> "def""#), Some(Object::Int(-1)));
    assert_eq!(run(r#""abc" <=> "abc""#), Some(Object::Int(0)));
    assert_eq!(run(r#""def" <=> "abc""#), Some(Object::Int(1)));
}

// ── Token display coverage ──────────────────────────────────────────────────

#[test]
fn lexer_tokens_display() {
    // Exercise the lexer token Display trait for new token types
    use metorex::lexer::Lexer;
    let tokens = Lexer::new("=~ !~ === ^ ||= &&=").tokenize();
    let displays: Vec<String> = tokens.iter().map(|t| format!("{}", t.kind)).collect();
    assert!(displays.contains(&"=~".to_string()));
    assert!(displays.contains(&"!~".to_string()));
    assert!(displays.contains(&"===".to_string()));
    assert!(displays.contains(&"^".to_string()));
    assert!(displays.contains(&"||=".to_string()));
    assert!(displays.contains(&"&&=".to_string()));
}
