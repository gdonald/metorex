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

greet = Greeter.new.method :greet
unbound_greet = Greeter.instance_method :greet
hello = Greeter.new.method :hello

p greet.parameters
p greet.arity
p unbound_greet.arity
p hello.owner
p hello.super_method

class Loud < Greeter
  def greet(name)
    super
  end
end

louder = Loud.new.method :greet
unbound_louder = Loud.instance_method :greet
p louder.super_method.owner
p unbound_louder.super_method.owner
p Object.new.method(:frozen?).super_method

loose = proc { |a, b = 1| }
strict = lambda { |a, b = 1| }
keyworded = proc { |a, k:, **rest| }
shaped = lambda { |a, k:, **rest| }

p loose.arity
p strict.arity
p keyworded.arity
p shaped.parameters
