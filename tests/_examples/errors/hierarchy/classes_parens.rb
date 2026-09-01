# The built-in exception hierarchy, rooted at Object the way every class is.

puts(Exception.superclass.to_s)
puts(Exception.class.to_s)

# Directly under Exception.
puts(NoMemoryError.superclass.to_s)
puts(ScriptError.superclass.to_s)
puts(SecurityError.superclass.to_s)
puts(SignalException.superclass.to_s)
puts(StandardError.superclass.to_s)
puts(SystemExit.superclass.to_s)
puts(SystemStackError.superclass.to_s)

# Nested branches.
puts(Interrupt.superclass.to_s)
puts(LoadError.superclass.to_s)
puts(EOFError.superclass.to_s)
puts(KeyError.superclass.to_s)
puts(StopIteration.superclass.to_s)
puts(ClosedQueueError.superclass.to_s)
puts(NoMethodError.superclass.to_s)
puts(FloatDomainError.superclass.to_s)
puts(FrozenError.superclass.to_s)
puts(UncaughtThrowError.superclass.to_s)
puts(FiberError.superclass.to_s)
puts(ThreadError.superclass.to_s)

# A rescue of a base class catches its descendants.
begin
  raise(KeyError, "missing")
rescue IndexError => error
  puts(error.class.to_s)
end

begin
  raise(FrozenError, "frozen")
rescue StandardError => error
  puts(error.is_a?(RuntimeError).to_s)
end
