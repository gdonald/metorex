p SyntaxError.new.path
p SyntaxError.new("unexpected end").path

begin
  eval "if true", nil, "speccing.rb"
rescue SyntaxError => error
  puts error.class
  p error.path
end

begin
  eval "if true"
rescue SyntaxError => error
  p error.path
end

begin
  require_relative "bad_assignment"
rescue SyntaxError => error
  puts error.class
  puts error.path.end_with? "errors/syntax_error/bad_assignment.rb"
end
