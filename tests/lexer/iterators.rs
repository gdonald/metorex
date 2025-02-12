// Iterator helper tests

use metorex::lexer::{Lexer, TokenKind};

// ===== Iterator Tests =====

#[test]
fn test_lexer_iterator_basic() {
    let lexer = Lexer::new("1 2 3");
    let tokens: Vec<_> = lexer.collect();

    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].kind, TokenKind::Int(1));
    assert_eq!(tokens[1].kind, TokenKind::Int(2));
    assert_eq!(tokens[2].kind, TokenKind::Int(3));
}

#[test]
fn test_lexer_iterator_for_loop() {
    let lexer = Lexer::new("x y z");
    let mut count = 0;

    for token in lexer {
        count += 1;
        assert!(matches!(token.kind, TokenKind::Ident(_)));
    }

    assert_eq!(count, 3);
}

#[test]
fn test_lexer_iterator_filter() {
    let lexer = Lexer::new("1 + 2 - 3");
    let operators: Vec<_> = lexer
        .filter(|t| {
            matches!(
                t.kind,
                TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash
            )
        })
        .collect();

    assert_eq!(operators.len(), 2);
    assert_eq!(operators[0].kind, TokenKind::Plus);
    assert_eq!(operators[1].kind, TokenKind::Minus);
}

#[test]
fn test_lexer_iterator_map() {
    let lexer = Lexer::new("x y z");
    let positions: Vec<_> = lexer.map(|t| t.position.column).collect();

    assert_eq!(positions.len(), 3);
    assert_eq!(positions[0], 1);
    assert_eq!(positions[1], 3);
    assert_eq!(positions[2], 5);
}

#[test]
fn test_lexer_tokenize_helper() {
    let lexer = Lexer::new("1 + 2");
    let tokens = lexer.tokenize();

    assert_eq!(tokens.len(), 4); // 1, +, 2, EOF
    assert_eq!(tokens[0].kind, TokenKind::Int(1));
    assert_eq!(tokens[1].kind, TokenKind::Plus);
    assert_eq!(tokens[2].kind, TokenKind::Int(2));
    assert_eq!(tokens[3].kind, TokenKind::EOF);
}

#[test]
fn test_lexer_peek_token() {
    let mut lexer = Lexer::new("42");

    // Peek should not consume
    let peeked = lexer.peek_token();
    assert_eq!(peeked.kind, TokenKind::Int(42));

    // Next should return the same token
    let next = lexer.next_token();
    assert_eq!(next.kind, TokenKind::Int(42));

    // After consuming, peek should return EOF
    let peeked_eof = lexer.peek_token();
    assert_eq!(peeked_eof.kind, TokenKind::EOF);
}
