# `detailed_message` names the class at the end of the first line, leaving the
# lines under it as they were written.
p RuntimeError.new("first line\nsecond line").detailed_message(highlight: false)
p RuntimeError.new("").detailed_message(highlight: false)
p StandardError.new("").detailed_message(highlight: false)

# A message of nil is no message at all, so the exception names itself.
p RuntimeError.new(nil).message
p RuntimeError.new.message

# `full_message` hands every keyword it was given on to `detailed_message`,
# `order:` aside.
trouble = RuntimeError.new("new error")
trouble.define_singleton_method(:detailed_message) { |**options| options.inspect }
p trouble.full_message(foo: "bar", order: :top, highlight: false).lines.first.include?("foo")

# An exception carrying no backtrace is reported against the line asking.
plain = RuntimeError.new("no trace")
p plain.backtrace
p plain.full_message(highlight: false, order: :top).end_with?("': no trace (RuntimeError)\n")

# `raise` takes a backtrace of its own and a `cause:` beside the message.
begin
  raise RuntimeError, "written", ["/dir/foo.rb:10:in `raising'"]
rescue => held
  p held.backtrace
end

begin
  raise RuntimeError.new("outer"), cause: RuntimeError.new("inner")
rescue => held
  p held.message
  p held.cause.message
end
