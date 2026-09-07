# The ways a set can be cut into subsets, and the way it opens up the sets
# it holds. A hash answers a lambda that reads one key.
words = Set["one", "two", "three", "four", "five"]

p words.classify { |word| word.length }
p words.divide { |word| word.length }.map { |group| group.to_a.sort }.sort
p Set[1, 3, 4, 6].divide { |left, right| (left - right).abs == 1 }.map { |group| group.to_a.sort }.sort
p words.classify.class

nested = Set[1, 2, Set[3, Set[4, 5]], 6]
p nested.flatten.to_a.sort
p nested.flatten.equal?(nested)

flattening = Set[1, Set[2, 3]]
p flattening.flatten!.to_a.sort
p Set[1, 2].flatten!

reader = { a: 1, b: 2 }.to_proc
p reader.lambda?
p reader.arity
p reader.call(:a)
p [:a, :b].map(&reader)
