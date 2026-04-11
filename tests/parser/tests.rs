use metorex::ast::{Expression, Statement};
use metorex::error::MetorexError;
use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn parse_source(source: &str) -> Result<Vec<Statement>, Vec<MetorexError>> {
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[test]
fn test_parse_integer_literal() {
    let result = parse_source("42");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::IntLiteral { value, .. } => assert_eq!(*value, 42),
            _ => panic!("Expected IntLiteral"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_parse_arithmetic() {
    let result = parse_source("1 + 2 * 3");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);
}

#[test]
fn test_parse_assignment() {
    let result = parse_source("x = 42");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Assignment { .. } => {}
        _ => panic!("Expected Assignment statement"),
    }
}

#[test]
fn test_parse_function_def() {
    let result = parse_source("def foo(x, y)\n  x + y\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::FunctionDef {
            name,
            parameters,
            body,
            ..
        } => {
            assert_eq!(name, "foo");
            assert_eq!(parameters.len(), 2);
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected FunctionDef statement"),
    }
}

#[test]
fn test_parse_class_def() {
    let result = parse_source("class Foo\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::ClassDef { name, .. } => {
            assert_eq!(name, "Foo");
        }
        _ => panic!("Expected ClassDef statement"),
    }
}

#[test]
fn test_parse_if_statement() {
    let result = parse_source("if true\n  42\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::If {
            then_branch,
            else_branch,
            ..
        } => {
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_none());
        }
        _ => panic!("Expected If statement"),
    }
}

#[test]
fn test_parse_while_loop() {
    let result = parse_source("while true\n  42\nend");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::While { body, .. } => {
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected While statement"),
    }
}

#[test]
fn test_parse_method_call() {
    let result = parse_source("obj.method(1, 2)");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::MethodCall {
                method, arguments, ..
            } => {
                assert_eq!(method, "method");
                assert_eq!(arguments.len(), 2);
            }
            _ => panic!("Expected MethodCall"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_parse_array_literal() {
    let result = parse_source("[1, 2, 3]");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Array { elements, .. } => {
                assert_eq!(elements.len(), 3);
            }
            _ => panic!("Expected Array"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_parse_hash_literal_with_fat_arrow() {
    let result = parse_source(r#"{"alice" => 30, "bob" => 25}"#);
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Dictionary { entries, .. } => {
                assert_eq!(entries.len(), 2);

                // Check first entry
                match &entries[0] {
                    (
                        Expression::StringLiteral { value: key, .. },
                        Expression::IntLiteral { value: val, .. },
                    ) => {
                        assert_eq!(key, "alice");
                        assert_eq!(*val, 30);
                    }
                    _ => panic!("Expected StringLiteral => IntLiteral"),
                }

                // Check second entry
                match &entries[1] {
                    (
                        Expression::StringLiteral { value: key, .. },
                        Expression::IntLiteral { value: val, .. },
                    ) => {
                        assert_eq!(key, "bob");
                        assert_eq!(*val, 25);
                    }
                    _ => panic!("Expected StringLiteral => IntLiteral"),
                }
            }
            _ => panic!("Expected Dictionary"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_parse_hash_literal_empty() {
    let result = parse_source("{}");
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Dictionary { entries, .. } => {
                assert_eq!(entries.len(), 0);
            }
            _ => panic!("Expected Dictionary"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_parse_hash_literal_mixed_types() {
    let result = parse_source(r#"{1 => "one", "two" => 2, true => nil}"#);
    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Statement::Expression { expression, .. } => match expression {
            Expression::Dictionary { entries, .. } => {
                assert_eq!(entries.len(), 3);
            }
            _ => panic!("Expected Dictionary"),
        },
        _ => panic!("Expected Expression statement"),
    }
}

fn parse_sym(code: &str) {
    let tokens = Lexer::new(code).tokenize();
    Parser::new(tokens).parse().expect("parse failed");
}

fn run_sym(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

// ── Keyword symbols ─────────────────────────────────────────────────────────

#[test]
fn symbol_keyword_class() {
    assert_eq!(
        run_sym(":class"),
        Some(Object::Symbol(std::rc::Rc::new("class".to_string())))
    );
}
#[test]
fn symbol_keyword_def() {
    assert_eq!(
        run_sym(":def"),
        Some(Object::Symbol(std::rc::Rc::new("def".to_string())))
    );
}
#[test]
fn symbol_keyword_if() {
    assert_eq!(
        run_sym(":if"),
        Some(Object::Symbol(std::rc::Rc::new("if".to_string())))
    );
}
#[test]
fn symbol_keyword_else() {
    assert_eq!(
        run_sym(":else"),
        Some(Object::Symbol(std::rc::Rc::new("else".to_string())))
    );
}
#[test]
fn symbol_keyword_end() {
    assert_eq!(
        run_sym(":end"),
        Some(Object::Symbol(std::rc::Rc::new("end".to_string())))
    );
}
#[test]
fn symbol_keyword_do() {
    assert_eq!(
        run_sym(":do"),
        Some(Object::Symbol(std::rc::Rc::new("do".to_string())))
    );
}
#[test]
fn symbol_keyword_nil() {
    assert_eq!(
        run_sym(":nil"),
        Some(Object::Symbol(std::rc::Rc::new("nil".to_string())))
    );
}
#[test]
fn symbol_keyword_true() {
    assert_eq!(
        run_sym(":true"),
        Some(Object::Symbol(std::rc::Rc::new("true".to_string())))
    );
}
#[test]
fn symbol_keyword_false() {
    assert_eq!(
        run_sym(":false"),
        Some(Object::Symbol(std::rc::Rc::new("false".to_string())))
    );
}
#[test]
fn symbol_keyword_return() {
    assert_eq!(
        run_sym(":return"),
        Some(Object::Symbol(std::rc::Rc::new("return".to_string())))
    );
}
#[test]
fn symbol_keyword_begin() {
    assert_eq!(
        run_sym(":begin"),
        Some(Object::Symbol(std::rc::Rc::new("begin".to_string())))
    );
}
#[test]
fn symbol_keyword_rescue() {
    assert_eq!(
        run_sym(":rescue"),
        Some(Object::Symbol(std::rc::Rc::new("rescue".to_string())))
    );
}
#[test]
fn symbol_keyword_ensure() {
    assert_eq!(
        run_sym(":ensure"),
        Some(Object::Symbol(std::rc::Rc::new("ensure".to_string())))
    );
}
#[test]
fn symbol_keyword_while() {
    assert_eq!(
        run_sym(":while"),
        Some(Object::Symbol(std::rc::Rc::new("while".to_string())))
    );
}
#[test]
fn symbol_keyword_for() {
    assert_eq!(
        run_sym(":for"),
        Some(Object::Symbol(std::rc::Rc::new("for".to_string())))
    );
}
#[test]
fn symbol_keyword_case() {
    assert_eq!(
        run_sym(":case"),
        Some(Object::Symbol(std::rc::Rc::new("case".to_string())))
    );
}
#[test]
fn symbol_keyword_when() {
    assert_eq!(
        run_sym(":when"),
        Some(Object::Symbol(std::rc::Rc::new("when".to_string())))
    );
}
#[test]
fn symbol_keyword_module() {
    assert_eq!(
        run_sym(":module"),
        Some(Object::Symbol(std::rc::Rc::new("module".to_string())))
    );
}
#[test]
fn symbol_keyword_include() {
    assert_eq!(
        run_sym(":include"),
        Some(Object::Symbol(std::rc::Rc::new("include".to_string())))
    );
}
#[test]
fn symbol_keyword_yield() {
    assert_eq!(
        run_sym(":yield"),
        Some(Object::Symbol(std::rc::Rc::new("yield".to_string())))
    );
}
#[test]
fn symbol_keyword_super() {
    assert_eq!(
        run_sym(":super"),
        Some(Object::Symbol(std::rc::Rc::new("super".to_string())))
    );
}
#[test]
fn symbol_keyword_lambda() {
    assert_eq!(
        run_sym(":lambda"),
        Some(Object::Symbol(std::rc::Rc::new("lambda".to_string())))
    );
}
#[test]
fn symbol_keyword_break() {
    assert_eq!(
        run_sym(":break"),
        Some(Object::Symbol(std::rc::Rc::new("break".to_string())))
    );
}
#[test]
fn symbol_keyword_next() {
    assert_eq!(
        run_sym(":next"),
        Some(Object::Symbol(std::rc::Rc::new("next".to_string())))
    );
}
#[test]
fn symbol_keyword_raise() {
    assert_eq!(
        run_sym(":raise"),
        Some(Object::Symbol(std::rc::Rc::new("raise".to_string())))
    );
}
#[test]
fn symbol_from_ivar() {
    assert_eq!(
        run_sym(":@name"),
        Some(Object::Symbol(std::rc::Rc::new("@name".to_string())))
    );
}
#[test]
fn symbol_from_cvar() {
    assert_eq!(
        run_sym(":@@count"),
        Some(Object::Symbol(std::rc::Rc::new("@@count".to_string())))
    );
}
#[test]
fn symbol_from_string_literal() {
    assert_eq!(
        run_sym(r#":"hello""#),
        Some(Object::Symbol(std::rc::Rc::new("hello".to_string())))
    );
}
#[test]
fn interpolated_symbol_dynamic() {
    let result = run_sym(r#"x = "name"; :"@#{x}""#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("@name".to_string())))
    );
}

// ── Token stream / general parser (from additional_tests) ───────────────────

#[test]
fn token_stream_peek_past_end_via_empty_parse() {
    parse_sym(""); // empty parse exercises peek-past-end
}

#[test]
fn parser_handles_multiple_newlines() {
    parse_sym("x = 1\n\n\ny = 2");
}

#[test]
fn parser_handles_comments_between_statements() {
    parse_sym("x = 1\n# comment\ny = 2");
}

#[test]
fn parse_dict_fat_arrow() {
    parse_sym(r#"h = { "a" => 1, "b" => 2 }"#);
}

#[test]
fn parse_grouped_expression() {
    parse_sym("x = (1 + 2) * 3");
}

#[test]
fn parse_symbol_literal_test() {
    parse_sym("x = :hello");
}

// ── Parenthesized assignment — (target = value) as expression ───────────────

#[test]
fn parenthesized_simple_assignment_as_expression() {
    let result = run_sym("(x = 5); x");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn parenthesized_ivar_assignment_as_expression() {
    let result = run_sym("class Foo\n  def bar\n    (@x = 42).to_s\n  end\nend\nFoo.new.bar");
    assert_eq!(result, Some(Object::string("42")));
}

#[test]
fn parenthesized_assignment_value_chain_with_method() {
    let result = run_sym("(y = [1, 2, 3]).length");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── def followed by a newline then (expression) must not eat ( as params ────

#[test]
fn def_newline_paren_expression_not_consumed_as_params() {
    let result = run_sym(
        "class Box\n  def put\n    (@v = 7)\n  end\n  def v\n    @v\n  end\nend\nb = Box.new\nb.put\nb.v",
    );
    assert_eq!(result, Some(Object::Int(7)));
}

// ── Leading :: constant access ─────────────────────────────────────────────

#[test]
fn leading_coloncolon_resolves_to_top_level_constant() {
    let result = run_sym("Foo = 99\n::Foo");
    assert_eq!(result, Some(Object::Int(99)));
}

// ── %w[] percent-word array literal ────────────────────────────────────────

#[test]
fn percent_w_brackets_splits_by_whitespace() {
    let result = run_sym("%w[alpha beta gamma].length");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn percent_w_bang_delimited() {
    let result = run_sym("%w!one two!.length");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn percent_w_empty_returns_empty_array() {
    let result = run_sym("%w[].length");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn percent_w_first_element_is_string() {
    let result = run_sym("%w[hello world].first");
    assert_eq!(result, Some(Object::string("hello")));
}

// ── Double-splat **expr in argument list ───────────────────────────────────

#[test]
fn double_splat_in_call_passes_through() {
    // `**h` in a call is accepted; downstream it behaves like passing h itself.
    let result = run_sym("def f(h)\n  h.class.to_s\nend\nh = {\"a\" => 1}\nf(**h)");
    assert_eq!(result, Some(Object::string("Hash")));
}

// ── return target = value (assignment in return value) ─────────────────────

#[test]
fn return_with_assignment_expression() {
    let result = run_sym("def f(flag)\n  return x = 99 if flag\n  -1\nend\nf(true)");
    assert_eq!(result, Some(Object::Int(99)));
}

// ── Array + Array concatenation ────────────────────────────────────────────

#[test]
fn array_plus_array_concatenates() {
    let result = run_sym("([1, 2] + [3, 4]).length");
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn array_plus_array_preserves_order() {
    let result = run_sym("([1, 2] + [3, 4])[2]");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── case/when does not introduce a scope — assignments leak out ────────────

#[test]
fn case_when_assignment_leaks_out() {
    let result = run_sym("x = 1\nn = nil\ncase x\nwhen 1\n  n = 10\nwhen 2\n  n = 20\nend\nn");
    assert_eq!(result, Some(Object::Int(10)));
}

// ── Ternary with ?-predicate method on true branch ─────────────────────────

#[test]
fn ternary_with_qmark_method_in_true_branch() {
    let result = run_sym(
        "class E\n  def failure?\n    true\n  end\nend\ne = E.new\ntrue ? e.failure? : e.failure?",
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Array#[start, length] two-argument slice ───────────────────────────────

#[test]
fn array_bracket_start_length_slice() {
    let result = run_sym("a = [1, 2, 3, 4, 5]\na[1, 3].length");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_bracket_start_length_contents() {
    let result = run_sym("a = [10, 20, 30, 40]\na[1, 2][0]");
    assert_eq!(result, Some(Object::Int(20)));
}

#[test]
fn array_bracket_negative_start() {
    let result = run_sym("a = [1, 2, 3, 4]\na[-2, 2][0]");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_bracket_out_of_range_returns_nil() {
    let result = run_sym("a = [1, 2, 3]\na[10, 2]");
    assert_eq!(result, Some(Object::Nil));
}

// ── Method-level rescue returns body value on success ──────────────────────

#[test]
fn method_level_rescue_returns_body_value_on_success() {
    let result = run_sym("def safe\n  42\nrescue => e\n  -1\nend\nsafe");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn method_level_rescue_catches_and_returns_rescue_value() {
    let result = run_sym("def safe\n  raise \"boom\"\nrescue => e\n  -1\nend\nsafe");
    assert_eq!(result, Some(Object::Int(-1)));
}

// ── return from inside .each propagates out of the method ─────────────────

#[test]
fn return_from_each_block_returns_from_method() {
    let result = run_sym(
        "def first_match\n  [1, 2, 3].each do |x|\n    return x if x > 1\n  end\n  -1\nend\nfirst_match",
    );
    assert_eq!(result, Some(Object::Int(2)));
}

// ── Hash missing key returns nil (Ruby semantics) ─────────────────────────

#[test]
fn hash_missing_key_returns_nil() {
    let result = run_sym(r#"h = {"a" => 1}; h["missing"]"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── bare `new` inside `def self.method` creates an instance ────────────────

#[test]
fn bare_new_in_class_method_creates_instance() {
    let result = run_sym(
        "class Foo\n  def initialize\n    @x = 42\n  end\n  def x\n    @x\n  end\n  def self.make\n    new\n  end\nend\nFoo.make.x",
    );
    assert_eq!(result, Some(Object::Int(42)));
}

// ── bare-id callee with args in method body dispatches with args ──────────

#[test]
fn bare_qmark_method_with_args_in_method_body() {
    let result = run_sym(
        "class C\n  def yield?(invert)\n    !invert\n  end\n  def check\n    yield?(false)\n  end\nend\nC.new.check",
    );
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── &block param is nil when method is called without a block ──────────────

#[test]
fn block_param_is_nil_when_no_block_given() {
    let result = run_sym("def f(&blk)\n  blk.nil?\nend\nf");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Operator symbols: :+, :-, :*, :==, etc. ───────────────────────────────

#[test]
fn symbol_bracket_index() {
    assert_eq!(
        run_sym(":[]"),
        Some(Object::Symbol(std::rc::Rc::new("[]".to_string())))
    );
}

#[test]
fn symbol_bracket_index_assign() {
    assert_eq!(
        run_sym(":[]="),
        Some(Object::Symbol(std::rc::Rc::new("[]=".to_string())))
    );
}

#[test]
fn symbol_plus() {
    assert_eq!(
        run_sym(":+"),
        Some(Object::Symbol(std::rc::Rc::new("+".to_string())))
    );
}

#[test]
fn symbol_minus() {
    assert_eq!(
        run_sym(":-"),
        Some(Object::Symbol(std::rc::Rc::new("-".to_string())))
    );
}

#[test]
fn symbol_star() {
    assert_eq!(
        run_sym(":*"),
        Some(Object::Symbol(std::rc::Rc::new("*".to_string())))
    );
}

#[test]
fn symbol_percent() {
    assert_eq!(
        run_sym(":%"),
        Some(Object::Symbol(std::rc::Rc::new("%".to_string())))
    );
}

#[test]
fn symbol_equal_equal() {
    assert_eq!(
        run_sym(":=="),
        Some(Object::Symbol(std::rc::Rc::new("==".to_string())))
    );
}

#[test]
fn symbol_bang_equal() {
    assert_eq!(
        run_sym(":!="),
        Some(Object::Symbol(std::rc::Rc::new("!=".to_string())))
    );
}

#[test]
fn symbol_less() {
    assert_eq!(
        run_sym(":<"),
        Some(Object::Symbol(std::rc::Rc::new("<".to_string())))
    );
}

#[test]
fn symbol_greater() {
    assert_eq!(
        run_sym(":>"),
        Some(Object::Symbol(std::rc::Rc::new(">".to_string())))
    );
}

#[test]
fn symbol_less_equal() {
    assert_eq!(
        run_sym(":<="),
        Some(Object::Symbol(std::rc::Rc::new("<=".to_string())))
    );
}

#[test]
fn symbol_greater_equal() {
    assert_eq!(
        run_sym(":>="),
        Some(Object::Symbol(std::rc::Rc::new(">=".to_string())))
    );
}

#[test]
fn symbol_spaceship() {
    assert_eq!(
        run_sym(":<=>"),
        Some(Object::Symbol(std::rc::Rc::new("<=>".to_string())))
    );
}

#[test]
fn symbol_shovel() {
    assert_eq!(
        run_sym(":<<"),
        Some(Object::Symbol(std::rc::Rc::new("<<".to_string())))
    );
}

#[test]
fn symbol_caret() {
    assert_eq!(
        run_sym(":^"),
        Some(Object::Symbol(std::rc::Rc::new("^".to_string())))
    );
}

#[test]
fn symbol_match_op() {
    assert_eq!(
        run_sym(":=~"),
        Some(Object::Symbol(std::rc::Rc::new("=~".to_string())))
    );
}

#[test]
fn symbol_not_match_op() {
    assert_eq!(
        run_sym(":!~"),
        Some(Object::Symbol(std::rc::Rc::new("!~".to_string())))
    );
}

#[test]
fn symbol_raise_keyword() {
    assert_eq!(
        run_sym(":raise"),
        Some(Object::Symbol(std::rc::Rc::new("raise".to_string())))
    );
}
