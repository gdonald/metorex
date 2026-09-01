# Every `Errno::EXXX` is a subclass of SystemCallError carrying the platform's
# own number in its `Errno` constant.

puts Errno::EINVAL.to_s
puts Errno::EINVAL.superclass.to_s
puts Errno::EINVAL::Errno.to_s
puts Errno::ENOENT::Errno.to_s
puts (Errno::EWOULDBLOCK::Errno == Errno::EAGAIN::Errno).to_s

invalid = Errno::EINVAL.new
puts invalid.class.to_s
puts invalid.errno.to_s
puts (Errno::EINVAL === invalid).to_s
puts (SystemCallError === invalid).to_s
puts (Errno::ENOENT === invalid).to_s

# `SystemCallError.new(message, errno)` answers the class that number names.
named = SystemCallError.new("boom", Errno::ENOENT::Errno)
puts named.class.to_s
puts named.message
puts (SystemCallError === named).to_s

# `===` is reachable by name as well as by operator.
puts Errno::EINVAL.__send__(:===, invalid).to_s

# A plain exception has no errno.
puts RuntimeError.new("x").errno.inspect

# Each Errno class carries the message its number stands for, with a custom
# message and location appended when given.
puts Errno::EINVAL.new.message
puts Errno::EINVAL.new("custom message").message
puts Errno::EINVAL.new("custom message", "location").message
puts Errno::ENOENT.new.message

# A subclass inherits the default message.
missing = Class.new(Errno::ENOENT)
puts missing.new("custom message").message
