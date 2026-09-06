# `Signal.signame` answers the name a number goes by, and nil for a number no
# signal uses.
puts(Signal.signame(0))
puts(Signal.signame(Signal.list["TERM"]))
p(Signal.signame(-1))

# The name a signal goes by wins over the older spelling of it.
puts(Signal.signame(Signal.list["ABRT"]))
puts(Signal.signame(Signal.list["CHLD"]))

# `Signal.list` carries CLD alongside CHLD, at the same number.
puts(Signal.list["CLD"] == Signal.list["CHLD"])

# Anything but an Integer is asked for `to_int`.
class SignalNumber
  def to_int
    0
  end
end

puts(Signal.signame(SignalNumber.new))

begin
  Signal.signame("hello")
rescue TypeError => error
  puts(error.message)
end

class NotANumber
  def to_int
    "not an int"
  end
end

begin
  Signal.signame(NotANumber.new)
rescue TypeError => error
  puts(error.message)
end
