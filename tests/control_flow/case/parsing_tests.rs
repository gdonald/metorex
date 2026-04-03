use metorex::lexer::Lexer;
use metorex::parser::Parser;

#[test]
fn test_parse_case_basic() {
    let source = r#"
case x
when 1
  puts "one"
when 2
  puts "two"
else
  puts "other"
end
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse basic case statement");
}

#[test]
fn test_parse_case_with_strings() {
    let source = r#"
case command
when "start"
  puts "Starting"
when "stop"
  puts "Stopping"
else
  puts "Unknown"
end
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse case with strings");
}

#[test]
fn test_parse_case_with_wildcard() {
    let source = r#"
case value
when 1
  puts "one"
when _
  puts "other"
end
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse case with wildcard");
}

#[test]
fn test_parse_case_no_else() {
    let source = r#"
case x
when 0
  puts "zero"
when 1
  puts "one"
end
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse case without else");
}

#[test]
fn test_parse_case_with_booleans() {
    let source = r#"
case flag
when true
  puts "enabled"
when false
  puts "disabled"
end
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse case with booleans");
}

#[test]
fn test_parse_case_with_nil() {
    let source = r#"
case value
when nil
  puts "no value"
else
  puts "has value"
end
"#;

    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse case with nil");
}
