# Two exceptions are equal when they share a class, a message, and a
# backtrace, so a copy equals its original.

one = ArgumentError.new
puts((one == one).to_s)
puts((one == one.dup).to_s)

puts((RuntimeError.new == RuntimeError.new).to_s)
puts((TypeError.new("message") == TypeError.new("message")).to_s)

# A different class, message, or backtrace makes them unequal.
puts((RuntimeError.new("message") == TypeError.new("message")).to_s)
puts((RuntimeError.new("message") == RuntimeError.new("other")).to_s)

traced = RuntimeError.new("message")
traced.set_backtrace(["a.rb:1"])
untraced = RuntimeError.new("message")
puts((traced == untraced).to_s)

matching = RuntimeError.new("message")
matching.set_backtrace(["a.rb:1"])
puts((traced == matching).to_s)

# Anything that is not an exception is never equal.
puts((ArgumentError.new == "").to_s)
puts((ArgumentError.new != RuntimeError.new).to_s)
