# The same run of eval written without parentheses.

class Cabinet
  def stock
    eval "class Shelf; end"
    Shelf.name
  end
end

p Cabinet.new.stock

p Kernel.eval "self"
p Kernel.respond_to? :eval
p method(:eval).call "2 * 3"

class Speaker
  def self.say
    yield
  end
end

begin
  eval("Speaker.say") { "from the block" }
rescue LocalJumpError => refused
  p refused.class
end

answer = -> do
  proc do
    eval "return :from_the_eval"
  end.call
  :never_reached
end.call
p answer

begin
  proc { eval "return :nowhere" }.call
rescue LocalJumpError => stranded
  p stranded.class
end

held = binding
eval "if false; counted = 1; end", held
p eval("counted", held)

source = <<SOURCE.b
# encoding: UTF-8
class Cabinet
  Widthπ = 12
end
SOURCE
p source.encoding
eval source
p Cabinet.constants(false).include? :"Widthπ"
p Cabinet::Widthπ

class Coerced
  def to_str
    "6 * 7"
  end
end

p eval Coerced.new

begin
  eval "1", proc {}
rescue TypeError => refused
  p refused.message
end
