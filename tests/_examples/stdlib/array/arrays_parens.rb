arr = [3, 1, 4, 1, 5, 9, 2, 6]
puts(arr.length)
puts(arr.size)

arr2 = [1, 2, 3]
arr2.push(4)
puts(arr2)

popped = arr2.pop
puts(popped)
puts(arr2)

arr3 = [1, 2, 3]
first = arr3.shift
puts(first)
puts(arr3)

arr3.unshift(0)
puts(arr3)

nums = [3, 1, 4, 1, 5, 9, 2, 6]
puts(nums.sort)

puts(nums.reverse)

doubled = [1, 2, 3].map { |n| n * 2 }
puts(doubled)

evens = [1, 2, 3, 4, 5, 6].select { |n| n % 2 == 0 }
puts(evens)

sum = [1, 2, 3, 4, 5].reduce(0) { |acc, n| acc + n }
puts(sum)

letters = ["a", "b", "c"]
puts(letters.join(", "))
puts(letters.join)
