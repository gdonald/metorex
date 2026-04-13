// Tests for compiler "not yet implemented" paths: super, scope resolution,
// if/unless expressions, case, magic constants, regex, defined?.

use metorex::compiler::Compiler;
use metorex::lexer::Lexer;
use metorex::parser::Parser;

// ── Unimplemented expression types ──────────────────────────────────────────

#[test]
fn compile_super_not_implemented() {
    let tokens = Lexer::new("super").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(
        err.contains("not yet implemented") || err.contains("Compilation"),
        "Error was: {}",
        err
    );
}

#[test]
fn compile_scope_resolution_not_implemented() {
    let tokens = Lexer::new("Foo::Bar").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(
        err.contains("not yet implemented") || err.contains("Compilation"),
        "Error was: {}",
        err
    );
}

#[test]
fn compile_case_not_yet_implemented() {
    let tokens = Lexer::new("x = case 1\nwhen 1\n  42\nend").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(
        err.contains("not yet implemented") || err.contains("Compilation"),
        "Error was: {}",
        err
    );
}

#[test]
fn compile_if_expression_not_implemented() {
    use metorex::ast::{Expression, Statement};
    use metorex::lexer::token::Position;

    let pos = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
    let if_expr = Expression::If {
        condition: Box::new(Expression::BoolLiteral {
            value: true,
            position: pos,
        }),
        then_branch: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos,
            },
            position: pos,
        }],
        elsif_branches: vec![],
        else_branch: None,
        position: pos,
    };
    let stmt = Statement::Expression {
        expression: if_expr,
        position: pos,
    };
    let compiler = Compiler::new();
    let result = compiler.compile(&[stmt]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not yet implemented"), "Error was: {}", err);
}

#[test]
fn compile_unless_expression_not_implemented() {
    use metorex::ast::{Expression, Statement};
    use metorex::lexer::token::Position;

    let pos = Position {
        line: 1,
        column: 1,
        offset: 0,
    };
    let unless_expr = Expression::Unless {
        condition: Box::new(Expression::BoolLiteral {
            value: true,
            position: pos,
        }),
        then_branch: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos,
            },
            position: pos,
        }],
        else_branch: None,
        position: pos,
    };
    let stmt = Statement::Expression {
        expression: unless_expr,
        position: pos,
    };
    let compiler = Compiler::new();
    let result = compiler.compile(&[stmt]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not yet implemented"), "Error was: {}", err);
}

// ── Magic constants not supported ───────────────────────────────────────────

#[test]
fn compile_magic_file_not_supported() {
    let tokens = Lexer::new("__FILE__").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(err.contains("not yet implemented") || err.contains("Compilation"));
}

#[test]
fn compile_magic_line_not_supported() {
    let tokens = Lexer::new("__LINE__").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(err.contains("not yet implemented") || err.contains("Compilation"));
}

#[test]
fn compile_magic_dir_not_supported() {
    let tokens = Lexer::new("__dir__").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(err.contains("not yet implemented") || err.contains("Compilation"));
}

#[test]
fn compile_regex_literal_not_supported() {
    let tokens = Lexer::new("/hello/").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(err.contains("not yet implemented") || err.contains("Compilation"));
}

#[test]
fn compile_defined_not_supported() {
    let tokens = Lexer::new("defined?(x)").tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse");
    let compiler = Compiler::new();
    let err = compiler.compile(&stmts).unwrap_err().to_string();
    assert!(err.contains("not yet implemented") || err.contains("Compilation"));
}
