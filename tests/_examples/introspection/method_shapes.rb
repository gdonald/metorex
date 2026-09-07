# What a Method object reports about the method it stands for.
module Greetings
  def hello(name)
    "hello #{name}"
  end
end

class Greeter
  include Greetings

  def greet(name, greeting = "Hi", *rest, punctuation:, times: 1, **options, &block)
    greeting
  end
end

p Greeter.new.method(:greet).parameters
p Greeter.new.method(:greet).arity
p Greeter.instance_method(:greet).arity
p Greeter.new.method(:hello).owner
p Greeter.new.method(:hello).super_method

class Loud < Greeter
  def greet(name)
    super
  end
end

p Loud.new.method(:greet).super_method.owner
p Loud.instance_method(:greet).super_method.owner
p Object.new.method(:frozen?).super_method

loose = proc { |a, b = 1| }
strict = lambda { |a, b = 1| }
keyworded = proc { |a, k:, **rest| }
shaped = lambda { |a, k:, **rest| }

p loose.arity
p strict.arity
p keyworded.arity
p shaped.parameters
