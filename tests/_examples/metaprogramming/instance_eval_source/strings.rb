# `instance_eval` takes source as well as a block. Either runs with `self`
# bound to the receiver, so both see its instance variables.

class Holder
  def initialize
    @value = 42
  end
end

holder = Holder.new
puts holder.instance_eval("@value").to_s
puts holder.instance_eval { @value }.to_s

# The block form yields the receiver.
puts "hola".instance_eval { |received| received.upcase }

# The block form takes no arguments of its own, and the source form takes
# between one and three.
begin
  "hola".instance_eval(4, 5) { |a, b| a + b }
rescue ArgumentError => error
  puts error.message
end

begin
  "hola".instance_eval
rescue ArgumentError => error
  puts error.message
end

begin
  "hola".instance_eval("1 + 1", "some file", 0, "bogus")
rescue ArgumentError => error
  puts error.message
end
