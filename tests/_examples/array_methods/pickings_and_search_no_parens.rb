# The ways elements can be picked out of an array, and the search that halves
# a sorted array rather than walking it.
numbers = [1, 2, 3, 4]

p numbers.combination(2).to_a
p numbers.combination(0).to_a
p numbers.combination(3).size
p numbers.permutation(2).to_a
p numbers.permutation.size
p numbers.repeated_combination(2).size
p [10, 11].repeated_combination(2).to_a
p [10, 11].repeated_permutation(2).to_a
p [10, 11].repeated_permutation(3).size

p [1, 2].product [3, 4], [5]
p [1, 2].product
p [1, 2].product []

collected = []
numbers.combination(2) { |pair| collected.push pair }
p collected.size
p numbers.combination(2) { |pair| pair }.equal? numbers

sorted = [0, 1, 3, 4]
p sorted.bsearch { |number| number >= 2 }
p sorted.bsearch { |number| number >= 9 }
p sorted.bsearch_index { |number| number >= 2 }
p sorted.bsearch { |number| number <=> 3 }
p sorted.bsearch { |number| number <=> 2 }
p [1].bsearch.class
