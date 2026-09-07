# A lambda written without parentheses takes defaults and keyword parameters,
# the same as one written with them.
greet = -> name = "world" { "hello #{name}" }
p greet.call
p greet.call("there")

spread = -> a, b, c, d = nil, e = nil { [a, b, c, d, e] }
p spread.call(1, 2, 3)

keyed = -> *rest, tag: :none { [rest, tag] }
p keyed.call(1, 2)
p keyed.call(1, tag: :marked)

parened = ->(width = 3) { width }
p parened.call

# A lambda takes its arguments the way a method does, so the count has to
# match what it declared.
pair = -> a, b { [a, b] }
begin
  pair.call(1)
rescue ArgumentError => error
  p error.message
end

# `**nil` says a method takes no keyword arguments at all.
def no_keywords(value, **nil)
  value
end
p no_keywords(7)

# A method handed over with & becomes the block, keeping its own arity.
class Recorder
  attr_reader :seen

  def initialize
    @seen = []
  end

  def record(pair)
    @seen.push(pair)
  end
end

recorder = Recorder.new
{ "a" => 1, "b" => 2 }.each(&recorder.method(:record))
p recorder.seen

# match hands the MatchData to a block and answers what the block answers.
p("hello".match(/l(l)o/) { |found| found[1] })
p(:hello.match(/l(l)o/) { |found| found[0] })
p "hello".match?(/l/, 3)
p "hello".match?(/h/, 1)

# Symbol has no constructor.
begin
  Symbol.new
rescue NoMethodError => error
  p error.message
end

# Two sets holding the same nested sets are equal whatever the order.
left = Set.new([Set.new([1, 2]), Set.new([3])])
right = Set.new([Set.new([3]), Set.new([2, 1])])
p(left == right)

# Integer answers zero? as its own rather than Numeric's.
p 42.method(:zero?).owner
