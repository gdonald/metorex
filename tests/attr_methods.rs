// Tests for attr_reader, attr_writer, and attr_accessor

use metorex::lexer::Lexer;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn execute_source(source: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        let is_eof = matches!(token.kind, metorex::lexer::TokenKind::EOF);
        tokens.push(token);
        if is_eof {
            break;
        }
    }

    let mut parser = Parser::new(tokens);
    let statements = parser.parse().map_err(|e| format!("{:?}", e))?;

    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements)
        .map_err(|e| format!("{:?}", e))?;

    Ok(())
}

#[test]
fn test_attr_reader_single() {
    let source = r#"
class Person
  attr_reader :name

  def initialize(name)
    @name = name
  end
end

p = Person.new("Alice")
puts p.name
"#;
    let result = execute_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_attr_reader_multiple() {
    let source = r#"
class Person
  attr_reader :name, :age, :city

  def initialize(name, age, city)
    @name = name
    @age = age
    @city = city
  end
end

p = Person.new("Bob", 25, "NYC")
puts p.name
puts p.age
puts p.city
"#;
    let result = execute_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_attr_writer_single() {
    let source = r#"
class Person
  attr_writer :name
  attr_reader :name

  def initialize
    @name = "Unknown"
  end
end

p = Person.new
p.name = "Charlie"
puts p.name
"#;
    let result = execute_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_attr_writer_multiple() {
    let source = r#"
class Person
  attr_writer :name, :age
  attr_reader :name, :age

  def initialize
    @name = "Unknown"
    @age = 0
  end
end

p = Person.new
p.name = "Dave"
p.age = 40
puts p.name
puts p.age
"#;
    let result = execute_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_attr_accessor_single() {
    let source = r#"
class Person
  attr_accessor :name

  def initialize(name)
    @name = name
  end
end

p = Person.new("Eve")
puts p.name
p.name = "Evelyn"
puts p.name
"#;
    let result = execute_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_attr_accessor_multiple() {
    let source = r#"
class Person
  attr_accessor :name, :age, :email

  def initialize(name, age, email)
    @name = name
    @age = age
    @email = email
  end
end

p = Person.new("Frank", 50, "frank@example.com")
puts p.name
p.name = "Francis"
puts p.name
puts p.age
p.age = 51
puts p.age
"#;
    let result = execute_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_attr_methods_with_inheritance() {
    let source = r#"
class Animal
  attr_accessor :name

  def initialize(name)
    @name = name
  end
end

class Dog < Animal
  attr_accessor :breed

  def initialize(name, breed)
    @name = name
    @breed = breed
  end
end

d = Dog.new("Buddy", "Golden Retriever")
puts d.name
puts d.breed
d.name = "Max"
d.breed = "Labrador"
puts d.name
puts d.breed
"#;
    let result = execute_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_attr_mixed_declarations() {
    let source = r#"
class Person
  attr_reader :id
  attr_writer :password
  attr_accessor :email

  def initialize(id)
    @id = id
    @password = "default"
    @email = "user@example.com"
  end

  def check_password(pwd)
    @password == pwd
  end
end

p = Person.new(123)
puts p.id
p.password = "secret"
puts p.check_password("secret")
puts p.email
p.email = "new@example.com"
puts p.email
"#;
    let result = execute_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_attr_reader_without_initialize() {
    let source = r#"
class Config
  attr_reader :debug

  @debug = false
end

c = Config.new
puts c.debug
"#;
    let result = execute_source(source);
    assert!(result.is_ok());
}

#[test]
fn test_attr_accessor_chaining() {
    let source = r#"
class Point
  attr_accessor :x, :y

  def initialize(x, y)
    @x = x
    @y = y
  end
end

p = Point.new(10, 20)
p.x = p.x + 5
p.y = p.y + 10
puts p.x
puts p.y
"#;
    let result = execute_source(source);
    assert!(result.is_ok());
}
