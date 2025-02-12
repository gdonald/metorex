// Higher-level lexer integration scenarios

use metorex::lexer::{InterpolationPart, Lexer, TokenKind};

// ===== Integration Tests =====

#[test]
fn test_lexer_full_expression() {
    let source = "def add(x, y)\n  x + y\nend";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens[0].kind, TokenKind::Def);
    assert_eq!(tokens[1].kind, TokenKind::Ident("add".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::LParen);
    assert_eq!(tokens[3].kind, TokenKind::Ident("x".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::Comma);
    assert_eq!(tokens[5].kind, TokenKind::Ident("y".to_string()));
    assert_eq!(tokens[6].kind, TokenKind::RParen);
    assert_eq!(tokens[7].kind, TokenKind::Newline);
    assert_eq!(tokens[8].kind, TokenKind::Ident("x".to_string()));
    assert_eq!(tokens[9].kind, TokenKind::Plus);
    assert_eq!(tokens[10].kind, TokenKind::Ident("y".to_string()));
    assert_eq!(tokens[11].kind, TokenKind::Newline);
    assert_eq!(tokens[12].kind, TokenKind::End);
}

#[test]
fn test_lexer_class_definition() {
    let source = "class Person\n  def initialize(@name, @age)\n    @@count += 1\n  end\nend";
    let mut lexer = Lexer::new(source);

    assert_eq!(lexer.next_token().kind, TokenKind::Class);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::Ident("Person".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::Newline);
    assert_eq!(lexer.next_token().kind, TokenKind::Def);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::Ident("initialize".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::LParen);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::InstanceVar("name".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::Comma);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::InstanceVar("age".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::RParen);
    assert_eq!(lexer.next_token().kind, TokenKind::Newline);
    assert_eq!(
        lexer.next_token().kind,
        TokenKind::ClassVar("count".to_string())
    );
    assert_eq!(lexer.next_token().kind, TokenKind::PlusEqual);
}

#[test]
fn test_lexer_conditionals_and_loops() {
    let source = "if x > 0\n  while y < 10\n    y += 1\n  end\nelse\n  x = 0\nend";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    // Verify key tokens exist
    assert!(tokens.iter().any(|t| t.kind == TokenKind::If));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::While));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Else));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::End));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::PlusEqual));
}

#[test]
fn test_lexer_array_and_hash_syntax() {
    let source = "[1, 2, 3] {x: 1, y: 2}";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens[0].kind, TokenKind::LBracket);
    assert_eq!(tokens[1].kind, TokenKind::Int(1));
    assert_eq!(tokens[2].kind, TokenKind::Comma);
    assert_eq!(tokens[3].kind, TokenKind::Int(2));
    assert_eq!(tokens[4].kind, TokenKind::Comma);
    assert_eq!(tokens[5].kind, TokenKind::Int(3));
    assert_eq!(tokens[6].kind, TokenKind::RBracket);
    assert_eq!(tokens[7].kind, TokenKind::LBrace);
    assert_eq!(tokens[8].kind, TokenKind::Ident("x".to_string()));
    assert_eq!(tokens[9].kind, TokenKind::Colon);
}

#[test]
fn test_lexer_string_interpolation_complex() {
    let source = r##""Hello #{name}, you are #{age} years old""##;
    let mut lexer = Lexer::new(source);
    let token = lexer.next_token();

    match token.kind {
        TokenKind::InterpolatedString(parts) => {
            assert_eq!(parts.len(), 5);
            assert_eq!(parts[0], InterpolationPart::Text("Hello ".to_string()));
            assert_eq!(parts[1], InterpolationPart::Expression("name".to_string()));
            assert_eq!(parts[2], InterpolationPart::Text(", you are ".to_string()));
            assert_eq!(parts[3], InterpolationPart::Expression("age".to_string()));
            assert_eq!(parts[4], InterpolationPart::Text(" years old".to_string()));
        }
        _ => panic!("Expected InterpolatedString"),
    }
}

#[test]
fn test_lexer_mixed_literals_and_operators() {
    let source = "42 + 3.14 * \"hello\" - true";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens[0].kind, TokenKind::Int(42));
    assert_eq!(tokens[1].kind, TokenKind::Plus);
    assert_eq!(tokens[2].kind, TokenKind::Float(3.14));
    assert_eq!(tokens[3].kind, TokenKind::Star);
    assert_eq!(tokens[4].kind, TokenKind::String("hello".to_string()));
    assert_eq!(tokens[5].kind, TokenKind::Minus);
    assert_eq!(tokens[6].kind, TokenKind::True);
}

#[test]
fn test_lexer_comments_in_code() {
    let source = "x = 1 # assign x\ny = 2 # assign y\n# final comment";
    let lexer = Lexer::new(source);
    let tokens: Vec<_> = lexer.collect();

    // Should have: x, =, 1, comment, newline, y, =, 2, comment, newline, comment
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Comment(c) if c == "assign x"))
    );
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Comment(c) if c == "assign y"))
    );
    assert!(
        tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Comment(c) if c == "final comment"))
    );
}
