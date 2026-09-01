# Modifying a frozen object raises FrozenError naming the class and the
# object, and the error carries the object as its receiver.

object = Object.new
object.freeze

begin
  def object.added
  end
rescue FrozenError => error
  puts error.class.to_s
  puts error.receiver.equal?(object).to_s
  puts error.message.start_with?("can't modify frozen Object: ").to_s
end

# A collection can be frozen too.
list = [1, 2]
list.freeze
puts list.frozen?.to_s

begin
  list << 3
rescue FrozenError => error
  puts error.message
end

puts list.inspect

# An unfrozen one is untouched.
mutable = [1]
mutable << 2
puts mutable.inspect
puts mutable.frozen?.to_s

# The receiver can be given by keyword.
target = Object.new
puts FrozenError.new("msg", receiver: target).receiver.equal?(target).to_s

# When `inspect` would itself modify the object, the message shows ...
tricky = Object.new
def tricky.inspect
  @seen = 1
end
def tricky.modify
  @seen = 2
end
tricky.freeze

begin
  tricky.modify
rescue FrozenError => error
  puts error.message
end
