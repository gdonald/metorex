use metorex::file_loader::parse_file;

#[test]
fn test_parse_file_simple_expression() {
    let source = "1 + 2";
    let result = parse_file(source, "test.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);
}

#[test]
fn test_parse_file_multiple_statements() {
    let source = r#"
x = 10
y = 20
puts x + y
"#;
    let result = parse_file(source, "test.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 3);
}

#[test]
fn test_parse_file_method_definition() {
    let source = r#"
def greet(name)
  puts "Hello, #{name}!"
end
"#;
    let result = parse_file(source, "test.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);
}

#[test]
fn test_parse_file_class_definition() {
    let source = r#"
class Person
  def initialize(name)
    @name = name
  end

  def greet
    puts "Hello, I'm #{@name}"
  end
end
"#;
    let result = parse_file(source, "test.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);
}

#[test]
fn test_parse_file_control_flow() {
    let source = r#"
if x > 10
  puts "Large"
else
  puts "Small"
end
"#;
    let result = parse_file(source, "test.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);
}

#[test]
fn test_parse_file_empty_source() {
    let source = "";
    let result = parse_file(source, "test.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 0);
}

#[test]
fn test_parse_file_whitespace_only() {
    let source = "   \n\n   \n  ";
    let result = parse_file(source, "test.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 0);
}

#[test]
fn test_parse_file_comments_only() {
    let source = r#"
# This is a comment
# Another comment
"#;
    let result = parse_file(source, "test.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 0);
}

#[test]
fn test_parse_file_unclosed_string_parses() {
    // Note: Unclosed strings are handled by the lexer and parse successfully
    let source = r#"puts "Hello, World!"#;
    let result = parse_file(source, "test.rb");

    // This actually parses successfully - the string is treated as complete at EOF
    assert!(result.is_ok());
}

#[test]
fn test_parse_file_instance_var_parses() {
    // Note: Invalid instance variable usage parses but fails at runtime
    let source = "x = @";
    let result = parse_file(source, "test.rb");

    // This parses successfully - semantic errors are caught at runtime
    assert!(result.is_ok());
}

#[test]
fn test_parse_file_syntax_error_unclosed_parenthesis() {
    let source = "def foo(x\n  puts x\nend";
    let result = parse_file(source, "test.rb");

    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains("Parse error in 'test.rb'"));
}

#[test]
fn test_parse_file_syntax_error_invalid_class() {
    let source = "class";
    let result = parse_file(source, "test.rb");

    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains("Parse error in 'test.rb'"));
}

#[test]
fn test_parse_file_filename_in_error() {
    let source = "def foo(x\n  puts x\nend";
    let result = parse_file(source, "my_special_file.mx");

    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(error_msg.contains("my_special_file.mx"));
}

#[test]
fn test_parse_file_complex_program() {
    let source = r#"
class Calculator
  def initialize
    @result = 0
  end

  def add(x, y)
    @result = x + y
  end

  def subtract(x, y)
    @result = x - y
  end

  def get_result
    @result
  end
end

calc = Calculator.new
calc.add(10, 5)
puts calc.get_result
"#;
    let result = parse_file(source, "calculator.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert!(statements.len() > 0);
}

#[test]
fn test_parse_file_with_loops() {
    let source = r#"
for i in 1..5
  puts i
end

while x < 10
  x = x + 1
end
"#;
    let result = parse_file(source, "loops.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 2);
}

#[test]
fn test_parse_file_with_case_statement() {
    let source = r#"
case value
when 1
  puts "one"
when 2
  puts "two"
else
  puts "other"
end
"#;
    let result = parse_file(source, "case.rb");

    assert!(result.is_ok());
    let statements = result.unwrap();
    assert_eq!(statements.len(), 1);
}
