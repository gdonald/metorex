# `instance_exec` runs a block with `self` bound to the receiver, passing its
# own arguments through to the block parameters.

class Counter
  def initialize
    @count = 7
  end
end

counter = Counter.new
puts counter.instance_exec { @count }.to_s
puts counter.instance_exec(3) { |extra| @count + extra }.to_s
puts counter.instance_exec(1, 2) { |first, second| first + second }.to_s

# Without a block there is nothing to yield to.
begin
  counter.instance_exec
rescue LocalJumpError => error
  puts error.message
end

# A singleton method on an immediate is refused, so a `def` in a block run
# against one raises rather than defining anything.
begin
  1.instance_exec { def never; end }
rescue TypeError => error
  puts error.message
end

begin
  :symbol.instance_exec { def never; end }
rescue TypeError => error
  puts error.message
end

puts Object.new.method(:instance_exec).arity.to_s
puts Object.new.method(:instance_eval).arity.to_s
