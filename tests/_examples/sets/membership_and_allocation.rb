# Set membership asks the element's hash and eql?, so two objects that agree
# on both are the same element even when they are different objects.
class Tagged
  def initialize(tag)
    @tag = tag
  end

  def hash
    42
  end

  def eql?(other)
    hash == other.hash
  end
end

first = Tagged.new(:first)
second = Tagged.new(:second)
holder = Set.new(["a", first])

p holder.include?(second)
p holder.member?(second)
p(holder === second)

# 1 and 1.0 are different elements, the way eql? tells them apart.
p Set.new([1, 1.0]).size
p(Set.new([1, 2, 3]).eql?(Set.new([3, 2, 1])))
p(Set.new([1, 2, 3]).eql?(Set.new([1.0, 2, 3])))

# A Set cannot be changed while a walk over it is open.
walked = Set.new([:a, :b])
walked.each do |element|
  begin
    walked << :c
  rescue RuntimeError => error
    p error.message.include?("iteration")
  end
end
p walked.to_a

# join renders each element as its to_s, so a Symbol contributes its name.
p Set.new([:a, :b, :c]).join
p Set.new([:a, :b]).join("-")
p [:x, :y].join

# allocate answers an empty one of the primitives it names.
p Array.allocate
p Hash.allocate
begin
  Proc.allocate
rescue TypeError => error
  p error.message
end
begin
  MatchData.allocate
rescue NoMethodError => error
  p error.message
end

# A Proc reports the methods it answers.
p Proc.public_instance_methods(false).include?(:call)
p Proc.public_instance_methods(false).include?(:eql?)

# Hash#replace answers the receiver, even when the new contents are written
# as keyword arguments.
target = { a: 1, b: 2 }
p target.replace(c: -1, d: -2).equal?(target)
p target

# transform_values! keeps what it changed when the block breaks out.
partial = { a: 1, b: 2, c: 3 }
partial.transform_values! do |value|
  break if value == 3
  100 + value
end
p partial

# A Range reports a size only when it steps from an Integer.
p (1..16).size
p ("a".."z").size
begin
  (1.0..16.0).size
rescue TypeError => error
  p error.message
end
