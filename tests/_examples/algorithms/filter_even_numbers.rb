numbers = [1, 2, 3, 4, 5, 6]
evens = numbers.select { |value| value % 2 == 0 }

puts evens
