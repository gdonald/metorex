# How a block binds what it is handed: a lone value spread across several
# parameters, required parameters on either side of optional ones and a
# splat, block-local names, and the parameter lists Ruby refuses. Written
# with as few parentheses as Ruby allows.

def hand(value)
  yield value
end

p(hand([1, 2]) { |first, *middle, last, extra| [first, middle, last, extra] })
p(hand([1, 2]) { |first = 5, second, third, fourth| [first, second, third, fourth] })
p(hand([1, 2, 3, 4]) { |a, b = 5, c = 6, d, e| [a, b, c, d, e] })
p(hand([1, 2, 3, 4, 5, 6]) { |a, b = 5, c = 6, d, e| [a, b, c, d, e] })
p(hand([1, 2, 3, 4]) { |a, b = 5, c = 6, *d, e, f| [a, b, c, d, e, f] })

class Pair
  def to_ary
    [:left, :right]
  end
end
p(hand(Pair.new) { |left, right, missing| [left, right, missing] })
p(hand(Pair.new) { |(left, right), rest| [left, right, rest] })

class Declines
  def to_ary
    nil
  end
end
declines = Declines.new
p(hand(declines) { |first, second| [first.equal?(declines), second] })

class Asked
  def respond_to?(name, include_all = false)
    puts "asked #{name} #{include_all}"
    super
  end

  def to_ary
    [1, 2]
  end
end
p(hand(Asked.new) { |first, second| first + second })

class Dynamic
  def method_missing(name, *arguments)
    name == :to_ary ? [3, 4] : super
  end

  def respond_to_missing?(name, include_private)
    name == :to_ary
  end
end
p(hand(Dynamic.new) { |first, second| first * second })

class Wrong
  def to_ary
    7
  end
end
begin
  hand(Wrong.new) { |first, second| first }
rescue TypeError => error
  p error.message
end

p(hand([1, 2]) { |_, _| _ })
p(proc { |same = same| same }.call)

outer = :outer
[1].each { |; outer| outer = :inner }
p outer
[1].each do |value; kept|
  p [value, kept]
end

["[1].each { |x, x| }", "-> (x, x) {}", "[1].each { |a; a| }", "[1].each { |a; b; c| }", "def lone; hand(1, &); end"].each do |source|
  begin
    eval source
  rescue SyntaxError
    puts "refused: #{source}"
  end
end

def forwards(&)
  "#{hand(1, &)} and #{[2].map(&).first}"
end
p forwards { |value| value * 10 }
