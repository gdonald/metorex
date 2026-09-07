# inject folds a walk, either through a block or through an operator named by
# a Symbol or a String.
class Numerous
  include Enumerable

  def initialize *list
    @list = list
  end

  def each
    @list.each { |value| yield value }
  end
end

numbers = Numerous.new 1, 2, 3, 4

p numbers.inject { |total, value| total + value }
p numbers.inject(10) { |total, value| total + value }
p numbers.inject :+
p numbers.inject "+"
p numbers.inject 10, :-
p numbers.reduce :*

p [1, 2, 3, 4].inject :+
p [1, 2, 3, 4].inject 10, :-

# A name that is neither a Symbol nor a String is asked for one through
# to_str, and anything else is a TypeError.
class Minus
  def to_str
    "-"
  end
end
p numbers.inject 10, Minus.new

begin
  numbers.inject 10, Object.new
rescue TypeError => error
  p error.message.include?("is not a symbol nor a string")
end

begin
  [1, 2].inject
rescue ArgumentError => error
  p error.message
end

# A block that grows the array while inject walks it reaches what it added.
grown = [1, 2, 3]
seen = []
grown.inject(nil) do |_, value|
  seen.push(value)
  grown.push(value + 10) if value < 4
  nil
end
p seen

# grep keeps the elements a pattern matches, grep_v the rest, and a block
# maps what is kept.
words = Numerous.new "apple", "banana", "cherry"
p words.grep(/an/)
p words.grep_v(/an/)
p words.grep(/rr/) { |word| word.upcase }
p Numerous.new(1, "two", :three, 4).grep Integer

# A parenthesized assignment names the receiver of a singleton definition.
class Matchers
  def odd_matcher
    def (@odd = Object.new).===(other)
      other.odd?
    end
    @odd
  end
end

matcher = Matchers.new.odd_matcher
p [1, 2, 3, 4].grep matcher
p [1, 2, 3, 4].grep_v matcher
