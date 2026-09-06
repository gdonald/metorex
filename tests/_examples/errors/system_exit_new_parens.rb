# `SystemExit.new` takes the status first, where true means a clean exit and
# false a failed one, and the message second.
exception = SystemExit.new(42, "message")
puts(exception.status.to_s)
puts(exception.message)
puts(exception.success?.to_s)

puts(SystemExit.new(true, "message").status.to_s)
puts(SystemExit.new(false, "message").status.to_s)

# Without a message the exception reports its class name.
puts(SystemExit.new(42).message)
puts(SystemExit.new(42).status.to_s)

# A lone message leaves the status at zero.
puts(SystemExit.new("message").status.to_s)
puts(SystemExit.new("message").message)
puts(SystemExit.new.status.to_s)
puts(SystemExit.new.message)

# A subclass carries a status the same way.
class CustomExit < SystemExit
end

puts(CustomExit.new(8).status.to_s)
puts(CustomExit.new(8).success?.to_s)
