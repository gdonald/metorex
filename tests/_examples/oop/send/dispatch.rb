# `send` and `__send__` name the method with a Symbol or a String, and say so
# when handed anything else.

class Greeter
  def greet
    "hello"
  end

  def self.build
    new
  end

  def collect(first, *rest)
    [first, *rest]
  end
end

puts Greeter.new.send(:greet)
puts Greeter.new.__send__("greet")
puts Greeter.send(:build).greet

puts Greeter.new.send(:collect, :one).inspect
puts Greeter.new.send(:collect, :one, :two).inspect

begin
  Greeter.new.send(42)
rescue TypeError => error
  puts error.message
end

begin
  Greeter.new.send
rescue ArgumentError => error
  puts error.message
end

begin
  Greeter.new.send(:missing)
rescue NoMethodError
  puts "NoMethodError for a name nothing defines"
end

puts BasicObject.instance_methods(false).sort.inspect
puts BasicObject.public_instance_methods(false).include?(:__send__).to_s

first_module = Module.new do
  def steps(taken)
    taken.push :first
  end
end

second_module = Module.new do
  def steps(taken = [])
    super(taken)
    taken.push :second
  end
end

layered = Class.new do
  include first_module
  include second_module
end

puts layered.new.steps.inspect
