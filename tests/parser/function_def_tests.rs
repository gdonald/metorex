// Tests for function definition parsing

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

#[test]
fn def_with_integer_name_is_parse_error() {
    let err = parse_err("def 123\n  nil\nend");
    assert!(err.contains("function name") || err.contains("Expected"));
}

#[test]
fn def_with_comment_only_body() {
    parse_ok("def silent\n  # nothing here\nend");
}

#[test]
fn def_with_variadic_parameter_parses_ok() {
    parse_ok("def sum(*args)\n  args\nend");
}

#[test]
fn def_block_param_with_invalid_name_error() {
    let err = parse_err("def foo(&123)\n  nil\nend");
    assert!(err.contains("parameter name") || err.contains("Expected"));
}

#[test]
fn variadic_param_with_non_ident_error() {
    let err = parse_err("def foo(*42)\nend");
    assert!(err.contains("parameter") || err.contains("Expected") || err.contains("Unexpected"));
}

#[test]
fn parser_function_multiline_body() {
    let result = run("def compute(a, b)\n  x = a + b\n  y = x * 2\n  y\nend\ncompute(3, 4)");
    assert_eq!(result, Some(Object::Int(14)));
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

// ── Method rescue/else/ensure ───────────────────────────────────────────────

#[test]
fn method_rescue_ensure_parsing() {
    parse_ok("def test\n  raise \"oops\"\nrescue\n  42\nensure\n  99\nend");
}

#[test]
fn method_rescue_else_parsing() {
    parse_ok("def safe\n  42\nrescue => e\n  -1\nelse\n  99\nend");
}

#[test]
fn method_rescue_else_ensure_parsing() {
    parse_ok("def full\n  42\nrescue => e\n  -1\nelse\n  99\nensure\n  0\nend");
}

// ── Singleton operator methods (def self.op) ─────────────────────────────────

#[test]
fn parse_def_self_dot_plus() {
    parse_ok("class X\n  def self.+(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_minus() {
    parse_ok("class X\n  def self.-(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_star() {
    parse_ok("class X\n  def self.*(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_starstar() {
    parse_ok("class X\n  def self.**(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_equalequal() {
    parse_ok("class X\n  def self.==(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_tripleequal() {
    parse_ok("class X\n  def self.===(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_bangequal() {
    parse_ok("class X\n  def self.!=(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_less() {
    parse_ok("class X\n  def self.<(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_greater() {
    parse_ok("class X\n  def self.>(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_lessequal() {
    parse_ok("class X\n  def self.<=(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_greaterequal() {
    parse_ok("class X\n  def self.>=(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_spaceship() {
    parse_ok("class X\n  def self.<=>(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_shovel() {
    parse_ok("class X\n  def self.<<(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_pipe() {
    parse_ok("class X\n  def self.|(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_ampersand() {
    parse_ok("class X\n  def self.&(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_match() {
    parse_ok("class X\n  def self.=~(other)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_brackets() {
    parse_ok("class X\n  def self.[](key)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_brackets_assign() {
    parse_ok("class X\n  def self.[]=(key, val)\n    1\n  end\nend");
}

#[test]
fn parse_def_self_dot_setter() {
    parse_ok("class X\n  def self.name=(val)\n    @name = val\n  end\nend");
}

// ── Function body with rescue/ensure ─────────────────────────────────────────

#[test]
fn parse_def_with_rescue() {
    parse_ok(
        r#"
def risky
  1 / 0
rescue => e
  0
end
"#,
    );
}

#[test]
fn parse_def_with_ensure() {
    parse_ok(
        r#"
def cleanup
  open_resource
ensure
  close_resource
end
"#,
    );
}

#[test]
fn parse_def_empty_body() {
    parse_ok("def noop\nend");
}

// ── Keyword method names (top-level def) ─────────────────────────────────────

#[test]
fn parse_def_next() {
    parse_ok("def next\n  1\nend");
}

#[test]
fn parse_def_include() {
    parse_ok("def include\n  1\nend");
}

#[test]
fn parse_def_extend() {
    parse_ok("def extend\n  1\nend");
}

#[test]
fn parse_def_module() {
    parse_ok("def module\n  1\nend");
}

#[test]
fn parse_def_class() {
    parse_ok("def class\n  1\nend");
}

#[test]
fn parse_def_raise() {
    parse_ok("def raise\n  1\nend");
}

#[test]
fn parse_def_begin() {
    parse_ok("def begin\n  1\nend");
}

#[test]
fn parse_def_lambda() {
    parse_ok("def lambda\n  1\nend");
}

#[test]
fn parse_def_yield_method() {
    parse_ok("def yield\n  1\nend");
}

#[test]
fn parse_def_return_method() {
    parse_ok("def return\n  1\nend");
}

#[test]
fn parse_def_break_method() {
    parse_ok("def break\n  1\nend");
}

#[test]
fn parse_def_defined() {
    parse_ok("def defined?\n  1\nend");
}

#[test]
fn parse_def_true_method() {
    parse_ok("def true\n  1\nend");
}

#[test]
fn parse_def_false_method() {
    parse_ok("def false\n  1\nend");
}

#[test]
fn parse_def_nil_method() {
    parse_ok("def nil\n  1\nend");
}

#[test]
fn parse_def_pipe_operator() {
    parse_ok("def |(other)\n  1\nend");
}

#[test]
fn parse_def_ampersand_operator() {
    parse_ok("def &(other)\n  1\nend");
}

#[test]
fn parse_def_shovel_operator() {
    parse_ok("def <<(other)\n  1\nend");
}

// ── Setter method name ──────────────────────────────────────────────────────

#[test]
fn parse_def_setter_method() {
    parse_ok("def name=(val)\n  @name = val\nend");
}

// ── Singleton method setter ─────────────────────────────────────────────────

#[test]
fn parse_def_self_dot_setter_method() {
    run(r#"
class Foo
  def self.value=(v)
    @val = v
  end
  def self.value
    @val
  end
end
Foo.value = 42
Foo.value
"#);
}

// ── lambda with a parenthesized argument list ────────────────────────────────

#[test]
fn lambda_with_a_block_pass_argument_parses_as_a_call() {
    parse_ok("lambda(&some_proc)");
}

#[test]
fn lambda_with_a_literal_block_still_parses_as_a_lambda() {
    let result = run("lambda { 42 }.call");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn lambda_with_a_do_block_still_parses_as_a_lambda() {
    let result = run("lambda do
  42
end.call");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── Keyword and operator method names after the singleton dot ────────────────

#[test]
fn singleton_def_accepts_a_keyword_method_name() {
    let result = run(r#"
class Widget
end
obj = Widget.new
def obj.class
  Integer
end
obj.class
"#);
    assert_eq!(result.map(|o| o.to_string()), Some("Integer".to_string()));
}

#[test]
fn singleton_def_on_instance_variable_accepts_a_keyword_method_name() {
    let result = run(r#"
class Holder
  def initialize
    @target = Object.new
    def @target.begin
      "opened"
    end
  end

  def target_begin
    @target.begin
  end
end
Holder.new.target_begin
"#);
    assert_eq!(result.map(|o| o.to_string()), Some("opened".to_string()));
}

#[test]
fn singleton_def_on_instance_variable_accepts_an_index_method_name() {
    let result = run(r#"
class Holder
  def initialize
    @target = Object.new
    def @target.[](index)
      index * 2
    end
  end

  def target_at(index)
    @target[index]
  end
end
Holder.new.target_at(21)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn singleton_def_on_parenthesized_receiver_accepts_a_keyword_method_name() {
    let result = run(r#"
class Widget
end
obj = Widget.new
def (obj).class
  Integer
end
obj.class
"#);
    assert_eq!(result.map(|o| o.to_string()), Some("Integer".to_string()));
}

// ── Singleton method with invalid token after dot ─────────────────────────────

#[test]
fn parse_def_self_dot_invalid_token_error() {
    // function.rs line 52: _ => error for unexpected token after '.' in singleton def
    let err = parse_err("def Foo.@x\n  1\nend");
    assert!(err.contains("method name") || err.contains("Expected") || err.contains("after"));
}

// Note: `def Foo./` and `def Foo.%` are not parseable because the lexer
// treats `/` after `.` as starting a regex literal and `%` as a percent
// literal. Those branches in function.rs (lines 27-28) are dead code.

// ── def (expr).method singleton (lines 119-138) ───────────────────────────

#[test]
fn parse_def_on_true_literal_singleton() {
    // `def (true).x; end` — singleton method on true, mapped to TrueClass.
    parse_ok(
        r#"
def (true).my_truthy
  :yes
end
"#,
    );
}

#[test]
fn parse_def_on_false_literal_singleton() {
    parse_ok(
        r#"
def (false).my_falsy
  :no
end
"#,
    );
}

#[test]
fn parse_def_on_nil_literal_singleton() {
    parse_ok(
        r#"
def (nil).my_nil_method
  :nil_result
end
"#,
    );
}

#[test]
fn parse_def_on_non_literal_expr_singleton() {
    // Non-literal expression (local var) → _singleton_receiver is None.
    parse_ok(
        r#"
x = Object.new
def (x).custom
  "per-instance"
end
"#,
    );
}

#[test]
fn parse_def_on_paren_non_ident_after_dot_errors() {
    // `def (x).` followed by non-Ident token → error at line 125.
    let err = parse_err(
        r#"
x = 42
def (x).@foo
  1
end
"#,
    );
    assert!(err.contains("method") || err.contains("Expected"));
}

// ── Singleton method body with immediate `end` (line 162 break) ──────────

#[test]
fn parse_def_with_empty_body_immediate_end() {
    parse_ok(
        r#"
def noop
end
"#,
    );
}

// ── `class << target = value` ────────────────────────────────────────────────

#[test]
fn singleton_class_on_an_assignment_stores_the_object() {
    let result = run(r#"
class << $receiver = Object.new
  def greet
    :hello
  end
end
$receiver.greet.inspect
"#);
    assert_eq!(result.map(|o| o.to_string()), Some(":hello".to_string()));
}

#[test]
fn singleton_class_on_an_instance_variable_assignment_stores_the_object() {
    let result = run(r#"
class Holder
  def build
    class << @target = Object.new
      def describe
        :built
      end
    end
    @target.describe
  end
end
Holder.new.build.inspect
"#);
    assert_eq!(result.map(|o| o.to_string()), Some(":built".to_string()));
}

#[test]
fn singleton_class_without_an_assignment_still_parses() {
    let result = run(r#"
plain = Object.new
class << plain
  def still_works
    :yes
  end
end
plain.still_works.inspect
"#);
    assert_eq!(result.map(|o| o.to_string()), Some(":yes".to_string()));
}

// ── %i symbol-array literals ─────────────────────────────────────────────────

#[test]
fn percent_i_builds_an_array_of_symbols() {
    let result = run("%i[alpha beta gamma].inspect");
    assert_eq!(
        result.map(|o| o.to_string()),
        Some("[:alpha, :beta, :gamma]".to_string())
    );
}

#[test]
fn percent_i_accepts_parenthesis_delimiters() {
    let result = run("%i(one two).inspect");
    assert_eq!(
        result.map(|o| o.to_string()),
        Some("[:one, :two]".to_string())
    );
}

#[test]
fn percent_i_elements_are_symbols() {
    let result = run("%i[alpha].first.class.name");
    assert_eq!(result.map(|o| o.to_string()), Some("Symbol".to_string()));
}

#[test]
fn percent_w_still_builds_an_array_of_strings() {
    let result = run("%w[alpha beta].first.class.name");
    assert_eq!(result.map(|o| o.to_string()), Some("String".to_string()));
}

#[test]
fn an_empty_percent_i_is_an_empty_array() {
    let result = run("%i[].inspect");
    assert_eq!(result.map(|o| o.to_string()), Some("[]".to_string()));
}
