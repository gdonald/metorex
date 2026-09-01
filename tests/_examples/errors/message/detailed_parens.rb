# `Exception#detailed_message` decorates the message with the class name, or
# stands in for an empty one. `#full_message` renders the whole thing the way
# an uncaught exception is printed.

puts(RuntimeError.new("new error").detailed_message)
puts(RuntimeError.new("").detailed_message)
puts(StandardError.new("").detailed_message)
puts(RuntimeError.new("new error").detailed_message(foo: true))

# An anonymous class has no name to decorate with.
anonymous = Class.new(RuntimeError)
puts(anonymous.new("message").detailed_message)
puts(anonymous.new("message").class.equal?(anonymous).to_s)
puts(anonymous.new("message").is_a?(RuntimeError).to_s)

reported = RuntimeError.new("Some runtime error")
reported.set_backtrace(["a.rb:1", "b.rb:2"])
puts(reported.full_message(highlight: false, order: :top))
puts(reported.full_message(highlight: false, order: :bottom))

# A class that defines its own `detailed_message` decides how it reads.
custom = Exception.new("new error")
def custom.detailed_message(**)
  "<prefix>#{message}<suffix>"
end
puts(custom.detailed_message)
puts(custom.full_message(highlight: false).include?("<prefix>new error<suffix>").to_s)
