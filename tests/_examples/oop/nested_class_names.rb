# A class nested in a class can inherit from the class holding it.
class Parser
  def name = "parser"

  class Lexer < ::Parser
  end

  class Scanner < Parser
  end
end
p(Parser::Lexer.superclass)
p(Parser::Scanner.superclass)
p(Parser::Lexer.new.name)

# Each class Struct.new answers is its own class.
Token = Struct.new(:type, :text)
Event = Struct.new(:name, :args)
token = Token.new(:ident, "x")
p(token.is_a?(Token))
p(token.is_a?(Event))
p(Event === token)
p(Class.new(Struct) === token)

# A method calls itself by name alone.
class Countdown
  def initialize(from)
    @from = from
  end

  def steps
    return [] if @from.zero?

    current = @from
    @from -= 1
    [current] + steps
  end
end
p(Countdown.new(4).steps)
