class Countdown
  def each
    yield 3
    yield 2
    yield 1
    :liftoff
  end
end

enumerator = Countdown.new.to_enum

puts(enumerator.next)
puts(enumerator.next)
puts(enumerator.next)

begin
  enumerator.next
rescue StopIteration => error
  puts(error.message)
  p error.result
end

enumerator.rewind
puts(enumerator.next)

class NoReturnValue
  def each
    yield :only
    nil
  end
end

exhausted = NoReturnValue.new.to_enum
exhausted.next

begin
  exhausted.next
rescue StopIteration => error
  p error.result
end

p(StopIteration.new("built by hand").result)
