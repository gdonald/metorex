# `Exception#to_s` is the message alone, and the class name when there is no
# message. The message argument is rendered with `to_s`, which any object may
# define.

class Exceptional < Exception
end

puts(Exceptional.new("something went wrong").to_s)
puts(Exceptional.new.to_s)
puts(Exception.new.to_s)
puts(RuntimeError.new("boom").message)

described = Object.new
def described.to_s
  "a described message"
end

puts(Exceptional.new(described).to_s)

begin
  raise("raised message")
rescue => error
  puts(error.to_s)
  puts(error.class.to_s)
  puts(error.message)
end

# A bare `raise` with a trailing comment is still the re-raise form.
def reraise
  begin
    raise() # note about this line
  rescue RuntimeError => error
    error.class.to_s
  end
end

puts(reraise)
