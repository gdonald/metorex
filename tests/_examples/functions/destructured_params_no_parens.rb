def pair_sum((left, right))
  left + right
end
puts pair_sum [3, 4]

def spread((first, second, *middle, last))
  [first, second, middle, last].inspect
end
puts spread [1, 2, 3, 4, 5]

def nested((outer, (inner_one, inner_two)))
  [outer, inner_one, inner_two].inspect
end
puts nested [1, [2, 3]]

def alongside prefix, (left, right), suffix
  [prefix, left, right, suffix].inspect
end
puts alongside 0, [1, 2], 3

def short_group((only_one, missing))
  [only_one, missing].inspect
end
puts short_group [9]

puts method(:pair_sum).arity
puts method(:pair_sum).parameters.inspect
