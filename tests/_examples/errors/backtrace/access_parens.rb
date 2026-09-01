# An exception that was never raised has no backtrace, so `#backtrace` is nil.
# Once raised it answers the same Array every time, and `set_backtrace`
# replaces it with the very Array it is handed.

unraised = RuntimeError.new("never raised")
puts(unraised.backtrace.inspect)

begin
  raise("raised here")
rescue RuntimeError => error
  first = error.backtrace
  puts(first.class.to_s)
  puts(first.equal?(error.backtrace).to_s)
  puts($!.class.to_s)
  puts($!.message)
  puts($@.class.to_s)
end

replaced = RuntimeError.new("replaced")
lines = ["one", "two"]
replaced.set_backtrace(lines)
puts(replaced.backtrace.inspect)
puts(replaced.backtrace.equal?(lines).to_s)

replaced.set_backtrace("single")
puts(replaced.backtrace.inspect)

replaced.set_backtrace(nil)
puts(replaced.backtrace.inspect)

begin
  replaced.set_backtrace([:not_a_string])
rescue TypeError => error
  puts(error.message)
end
