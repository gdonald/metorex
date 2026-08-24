puts 5.then { |n| n + 1 }
puts 5.yield_self { |n| n * 2 }
puts 5.tap { |n| puts n }

puts "value".then { |text| text.upcase }

numbers = [1, 2, 3]
puts numbers.yield_self { |list| list.length }

puts 7.then { 42 }
