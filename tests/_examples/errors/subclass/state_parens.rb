# An exception subclass runs its own `initialize` and carries the instance
# variables it sets. `dup` copies that state and leaves the singleton class
# behind.

class Detailed < StandardError
  attr_reader :code

  def initialize(message = nil)
    super
    @code = 42
  end

  def initialize_copy(other)
    super
    @copied = true
  end

  def copied?
    @copied == true
  end
end

original = Detailed.new("first failure")
puts(original.message)
puts(original.code.to_s)
puts(original.copied?.to_s)

def original.only_mine
  :mine
end

puts(original.only_mine.inspect)

copy = original.dup
puts(copy.message)
puts(copy.code.to_s)
puts(copy.copied?.to_s)
puts(copy.respond_to?(:only_mine).to_s)

# The copy keeps the backtrace and the cause, sharing the backtrace Array.
begin
  begin
    raise(StandardError, "the cause")
  rescue StandardError
    raise(Detailed, "the consequence")
  end
rescue Detailed => raised
  trace = raised.backtrace
  duplicated = raised.dup
  puts(duplicated.message)
  puts(duplicated.cause.message)
  puts(duplicated.backtrace.equal?(trace).to_s)
end
