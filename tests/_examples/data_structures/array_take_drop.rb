numbers = [1, 2, 3, 4]

p numbers.take 2
p numbers.drop 2
p numbers.take 0
p numbers.drop 9
p numbers.take 9

begin
  numbers.take(-1)
rescue ArgumentError => error
  puts error.message
end

begin
  numbers.drop(-1)
rescue ArgumentError => error
  puts error.message
end
