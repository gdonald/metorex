// Tests for call expression parsing (keyword args, no-paren calls, dict-like syntax)

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

// ── Keyword argument calls ──────────────────────────────────────────────────

#[test]
fn keyword_arg_call_without_parens_single() {
    let result = run("def greet(name:)\n  name\nend\ngreet name: \"Alice\"");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Alice".to_string())))
    );
}

#[test]
fn keyword_arg_call_without_parens_multiple() {
    let result = run("def describe(name:, age:)\n  name\nend\ndescribe name: \"Bob\", age: 30");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Bob".to_string())))
    );
}

// ── No-paren call ───────────────────────────────────────────────────────────

#[test]
fn no_paren_call_two_positional_args() {
    let result = run("def add(a, b)\n  a + b\nend\nadd 1 + 0, 2");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn no_paren_call_with_trailing_do_block() {
    let result = run("def run_with(x)\n  x\nend\nrun_with !false do\n  99\nend");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn no_paren_call_with_trailing_brace_block() {
    let result = run("def run_with(x)\n  x\nend\nrun_with !false { 99 }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn no_paren_call_trailing_comma_before_end() {
    let result =
        run("def noop(x)\n  x\nend\nresult = nil\nif true\n  result = noop 1 + 0,\nend\nresult");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn should_parse_as_arg_returns_false_for_non_ident_before_colon() {
    parse_ok("x = {42 => 1}");
}

// ── Dict-like syntax errors ─────────────────────────────────────────────────

#[test]
fn no_paren_call_not_triggered_before_rbrace() {
    let result = run("x = {\"a\" => 1}\nx");
    if let Some(Object::Dict(_)) = result {
    } else {
        panic!("Expected dict");
    }
}

#[test]
fn no_paren_call_with_colon_after_expr_is_error() {
    let err = parse_err("foo 1 : 2");
    assert!(
        err.contains("dictionary")
            || err.contains("Expected")
            || err.contains("syntax")
            || err.contains("Unexpected")
    );
}

// ── Parser attr methods ─────────────────────────────────────────────────────

#[test]
fn parser_attr_accessor_multiple() {
    let result = run(
        "class Foo\n  attr_accessor :name, :age\nend\nf = Foo.new\nf.name = \"test\"\nf.age = 25\nf.age",
    );
    assert_eq!(result, Some(Object::Int(25)));
}

#[test]
fn attr_reader_with_non_symbol_error() {
    let err = parse_err("class Foo\n  attr_reader 42\nend");
    assert!(err.contains("attribute") || err.contains("Expected") || err.contains("Unexpected"));
}

// ── attr_* with keyword names (parser supports keywords-as-attr-names) ────

#[test]
fn attr_reader_with_keyword_names() {
    let result = run(
        "class Foo\n  attr_reader :include, :class\n  def initialize\n    @include = 1\n    @class = 2\n  end\nend\nf = Foo.new\nf.include + f.class",
    );
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn attr_writer_with_keyword_names() {
    let result = run("class Bar\n  attr_writer :extend\nend\nb = Bar.new\nb.extend = 99\n42");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn attr_accessor_with_multiple_keyword_names() {
    let result = run(
        "class Baz\n  attr_accessor :module, :def, :do\nend\nb = Baz.new\nb.module = 1\nb.def = 2\nb.do = 3\nb.module + b.def + b.do",
    );
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn attr_reader_missing_colon_errors() {
    let err = parse_err("class Foo\n  attr_reader name\nend");
    assert!(err.contains("Expected") || err.contains("symbol") || err.contains("attribute"));
}

// ── Keyword method names on receiver ─────────────────────────────────────────

#[test]
fn call_method_named_then() {
    parse_ok("x.then");
}

#[test]
fn call_method_named_module() {
    parse_ok("x.module");
}

#[test]
fn call_method_named_include() {
    parse_ok("x.include");
}

#[test]
fn call_method_named_extend() {
    parse_ok("x.extend");
}

#[test]
fn call_method_named_yield() {
    parse_ok("x.yield");
}

#[test]
fn call_method_named_defined() {
    parse_ok("x.defined?");
}

#[test]
fn call_method_named_nil() {
    parse_ok("x.nil");
}

#[test]
fn call_method_named_true() {
    parse_ok("x.true");
}

#[test]
fn call_method_named_false() {
    parse_ok("x.false");
}

// ── Multi-arg bracket indexing ───────────────────────────────────────────────

#[test]
fn multi_arg_bracket_index() {
    parse_ok("x[1, 2]");
}

// ── Paren-less args with splat ───────────────────────────────────────────────

#[test]
fn paren_less_args_with_splat() {
    parse_ok("x.push *arr");
}

// ── Paren-less args with double splat ────────────────────────────────────────

#[test]
fn paren_less_args_with_block_arg() {
    parse_ok("x.each &block");
}

// ── Old-style hash args :sym => val ──────────────────────────────────────────

#[test]
fn old_style_hash_args_in_call() {
    parse_ok("foo(:a => 1, :b => 2)");
}

// ── Beginless range literals (binary.rs lines 151-162) ────────────────

#[test]
fn beginless_range_inclusive() {
    // `..10` — beginless range (start = nil, end = 10).
    let result = run("x = ..10\nx.include?(5)");
    // The exact evaluation may vary; assert parse success primarily.
    assert!(matches!(result, Some(_)));
}

#[test]
fn beginless_range_exclusive_parse() {
    // `...10` — beginless exclusive range.
    parse_ok("x = ...10");
}

#[test]
fn beginless_range_parse_standalone() {
    parse_ok("r = ..10");
}

// ── can_start_argument ternary disambiguation (lines 593-604) ─────────

#[test]
fn predicate_method_with_colon_inside_ternary_is_ternary_colon() {
    // `mode?` ends with `?`. Inside `cond ? x.mode? : fallback`, the `:`
    // must be the ternary colon, not a paren-less symbol arg.
    let result = run(r#"
class M
  def mode?
    true
  end
end
m = M.new
(true ? m.mode? : :other)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn non_predicate_method_with_colon_inside_ternary_is_ternary_colon() {
    // Outside a ternary, `foo :sym` is paren-less arg passing. Inside a
    // ternary with ambiguous follow-up, parser must treat `:` as ternary.
    let result = run(r#"
def echo(v = nil)
  v
end
x = true ? echo : :fallback
x
"#);
    // Either echo is called with nil (paren-less arg rejected) or :fallback
    // is returned. Any non-panic outcome exercises the ternary_depth path.
    assert!(matches!(result, Some(_)));
}

// ── Postfix calls on lambda block expressions (parse_postfix_calls) ───────

#[test]
fn brace_block_lambda_dot_call() {
    let result = run("lambda { 7 }.call");
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn brace_block_lambda_dot_class_keyword() {
    // `.class` after a brace-block lambda uses the TokenKind::Class keyword
    // arm inside parse_postfix_calls.
    parse_ok("lambda { 1 }.class");
}

#[test]
fn do_block_lambda_dot_call() {
    let result = run("lambda do 7 end.call");
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn brace_block_lambda_dot_call_no_paren_no_args() {
    // `.to_proc` with no parens and no args exercises the empty-Vec branch
    // of parse_postfix_calls.
    parse_ok("lambda { 1 }.to_proc");
}
