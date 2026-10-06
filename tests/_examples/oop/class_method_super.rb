# `super` in a class method reaches the next class method up the chain,
# and from an override of `allocate` or `new` reaches the core library's.
class Base
  def self.label = "base"
end

class Middle < Base
  def self.label = "middle " + super
end

class Leaf < Middle
  def self.label = "leaf " + super
end

p(Leaf.label)
p(Middle.label)

class Counted
  def self.allocate
    @made = (@made || 0) + 1
    super
  end

  def self.made = @made
end
p(Counted.allocate.class)
p(Counted.made)

class Greeting
  attr_reader :name, :reply

  def initialize(name, &reply)
    @name = name
    @reply = reply
  end

  def self.new(name, &reply)
    super(name.upcase, &reply)
  end
end
greeting = Greeting.new("ann") { "hello" }
p([greeting.class, greeting.name, greeting.reply.call])

class Farewell < Greeting
  def self.new(*arguments, &reply) = super
end
farewell = Farewell.new("bo") { "goodbye" }
p([farewell.class, farewell.name, farewell.reply.call])
