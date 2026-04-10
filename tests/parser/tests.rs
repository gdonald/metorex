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
