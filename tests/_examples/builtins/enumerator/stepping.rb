# An Enumerator steps through the values a method yields, one at a time.

class Steps
  def each
    yield "a"
    yield "b"
  end

  def upto(limit)
    count = 1
    while count <= limit
      yield count
      count = count + 1
    end
  end

  def pairs
    yield 1, 2
    yield 3, 4
  end
end

steps = Steps.new

letters = steps.to_enum
puts letters.to_a.inspect
puts letters.next
puts letters.next

begin
  letters.next
rescue StopIteration => error
  puts error.message
end

puts letters.rewind.peek

puts steps.enum_for(:upto, 3).to_a.inspect
puts steps.to_enum(:pairs).to_a.inspect

collected = []
steps.to_enum(:each).each { |value| collected.push value }
puts collected.inspect

object = Object.new
one = object.then
puts one.class.to_s
puts one.size.to_s
puts one.first.equal?(object).to_s
puts object.yield_self.peek.equal?(object).to_s
puts object.is_a?(Enumerable).to_s
puts one.is_a?(Enumerable).to_s
