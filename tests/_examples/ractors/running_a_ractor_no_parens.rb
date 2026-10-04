# A Ractor runs its block with `self` as the new Ractor and the arguments it
# was given copied in. `value` and `join` wait for the block to end, and what
# the block raised comes back as a Ractor::RemoteError whose cause it is.

Warning[:experimental] = false

p Ractor.current
p Ractor.main == Ractor.current
p Ractor.main?
p Ractor.count

adder = Ractor.new(1, 2, name: "adder") do |first, second|
  [self.class, self == Ractor.current, Ractor.main?, first + second, Ractor.current.name]
end
p adder.name
p adder.value
p adder.inspect.sub(/ \S+:\d+ /, " WRITTEN_AT ")

Thread.report_on_exception = false
failing = Ractor.new { raise ArgumentError, "bad" }
begin
  failing.join
rescue Ractor::RemoteError => trouble
  p [trouble.message, trouble.cause.class, trouble.cause.message, trouble.ractor == failing]
end

finished = Ractor.new { :done }
p finished.join == finished
p finished.value

text = +"plain text"
copied = Ractor.new(text) { |held| held.object_id }
p copied.value == text.object_id
p Ractor.new(:shared) { |held| held }.value

p Thread.new { Ractor.current }.value == Ractor.main
p Ractor.new { Thread.new { Ractor.current }.value == Ractor.current }.value
p Ractor.count

begin
  Ractor.new(name: 5) { }
rescue TypeError => refused
  p refused.message
end

begin
  Ractor.new
rescue ArgumentError => refused
  p refused.message
end

p Ractor::RemoteError.ancestors.take(4)
p [Ractor::ClosedError.superclass, Ractor::IsolationError.superclass, Ractor::MovedError.superclass]
