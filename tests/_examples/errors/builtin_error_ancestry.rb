# Ruby defines no ValueError, and every built-in error class reaches the
# StandardError a program names through its ancestors, which is how a rescue
# of StandardError places it.
p(defined?(ValueError))
errors = [ZeroDivisionError, TypeError, ArgumentError, KeyError, StopIteration, FrozenError, IOError, EOFError, Errno::ENOENT]
p(errors.all? { |error| error.ancestors.include?(StandardError) })
caught = errors.map do |error|
  raise error
rescue StandardError => raised
  raised.class
end
p(caught == errors)
begin
  raise NotImplementedError
rescue StandardError
  p(:not_reached)
rescue ScriptError => raised
  p(raised.class)
end
