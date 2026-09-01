puts(Signal.list["INT"] == 2)
puts(Signal.list.key?("TERM"))

interrupt = Interrupt.new
puts(interrupt.signo == Signal.list["INT"])
puts(interrupt.signm)

named = Interrupt.new("shutting down")
puts(named.signm)
puts(named.signo == Signal.list["INT"])

signal = SignalException.new(:TERM)
puts(signal.signm)
puts(signal.signo == Signal.list["TERM"])

previous = Signal.trap(:INT, :SIG_DFL)
puts(previous)

begin
  Process.kill(:INT, Process.pid)
rescue Interrupt => error
  puts(error.signo == Signal.list["INT"])
  puts(error.message)
end

begin
  Process.kill(:TERM, Process.pid)
rescue SignalException => error
  puts(error.signm)
end

Signal.trap(:USR1, :SIG_IGN)
puts(Process.kill(:USR1, Process.pid))

Signal.trap(:USR2) { |number| puts("handled #{number == Signal.list["USR2"]}") }
Process.kill(:USR2, Process.pid)

begin
  Process.kill(:NOPE, Process.pid)
rescue ArgumentError => error
  puts(error.message)
end
