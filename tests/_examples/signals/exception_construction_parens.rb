by_number = SignalException.new(Signal.list["INT"])
puts(by_number.signm)
puts(by_number.message)
puts(by_number.signo == Signal.list["INT"])

puts(SignalException.new("INT").signm)
puts(SignalException.new("SIGINT").signm)
puts(SignalException.new(:TERM).signm)
puts(SignalException.new(:SIGTERM).signm)

named = SignalException.new(Signal.list["INT"], "custom name")
puts(named.signm)
puts(named.message)
puts(named.signo == Signal.list["INT"])

def rejected
  yield
rescue ArgumentError => error
  error.message
end

puts(rejected { SignalException.new(100000) })
puts(rejected { SignalException.new("NONEXISTENT") })
puts(rejected { SignalException.new(:NONEXISTENT) })
puts(rejected { SignalException.new(Object.new) })
puts(rejected { SignalException.new("INT", "name") })

interrupt = Interrupt.new("still a message")
puts(interrupt.signm)
puts(interrupt.signo == Signal.list["INT"])
