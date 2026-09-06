# `SystemCallError.new` takes the number in the second position, or in the
# first when it is the only argument.
puts(SystemCallError.new(Errno::EINVAL::Errno).class.to_s)
puts(SystemCallError.new(Errno::EINVAL::Errno).message)
puts(SystemCallError.new("custom message", Errno::EINVAL::Errno).message)
puts(SystemCallError.new("custom message", Errno::EINVAL::Errno, "location").message)
puts(SystemCallError.new(nil, Errno::EINVAL::Errno).message)

# A number no Errno class names stays a SystemCallError, and still answers it.
unknown = SystemCallError.new("custom message", 2**24)
puts(unknown.class.to_s)
puts(unknown.errno.to_s)
# The text an unnamed number reads as comes from the C library, which words it
# differently on each platform, so only the custom half is printed here.
puts(unknown.message.end_with?(" - custom message").to_s)

# Without a number there is no errno at all.
puts(SystemCallError.new("message").errno.inspect)
puts(SystemCallError.new("message").message)

# The number is copied along with the rest of the exception.
puts(SystemCallError.new("message", 42).dup.errno.to_s)

# A Float or a real Complex is truncated to the number it stands for.
puts(SystemCallError.new("custom message", 2.9).class.to_s)
puts(SystemCallError.new("custom message", Complex(2.9, 0)).class.to_s)

# Ruby reports a variadic arity for the native initialize.
puts(SystemCallError.instance_method(:initialize).arity.to_s)

begin
  SystemCallError.new
rescue ArgumentError => error
  puts(error.class.to_s)
end

begin
  SystemCallError.new(:foo, 1)
rescue TypeError => error
  puts(error.message)
end

begin
  SystemCallError.new("custom message", "bar")
rescue TypeError => error
  puts(error.message)
end

begin
  SystemCallError.new("custom message", Complex(2.9, 1))
rescue RangeError => error
  puts(error.message)
end
