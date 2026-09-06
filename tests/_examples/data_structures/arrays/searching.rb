# The search methods ask each element whether it is `==` to what they were
# given, so an object that defines `==` decides for itself.
class AnyEven
  def ==(other)
    other.even?
  end
end

numbers = [1, 3, 4, 5]
p numbers.include? AnyEven.new
p numbers.index AnyEven.new
p numbers.rindex 5
p numbers.assoc 1
pairs = [[1, :a], [2, :b]]
p pairs.assoc 2
p pairs.rassoc :a

# Without a block or an argument these answer an Enumerator rather than
# walking the array.
p numbers.each_index.class.to_s
p numbers.each_index.to_a
p numbers.delete_if.class.to_s

# `delete` removes every element equal to the object, and runs its block when
# nothing matched.
letters = ["a", "b", "a"]
p letters.delete "a"
p letters
p letters.delete("z") { "not found" }

# `delete_if` moves the survivors forward and cuts the array at the end, so it
# keeps its length while the block runs.
counted = [1, 2, 3]
counted.delete_if do |element|
  counted.length
end
p counted

# `first` and `last` take a count, and `concat` takes anything that answers
# `to_ary`.
p numbers.first 2
p numbers.last 2
one = [1]
p one.concat [2, 3]
pair = [1, 2]
p pair.intersect? [2, 5]
