# Test coverage for stdlib methods
# Exercises methods and paths that need more test coverage

# === Float methods ===
puts 3.14.abs
puts(-3.14.abs)
puts 3.7.ceil
puts 3.2.floor
puts 3.5.round(0)
puts 3.14.to_i
puts 3.14.to_s

# === Integer conversion methods ===
puts 42.to_f
puts 42.to_i
puts 42.to_s

# === String methods ===
puts "hello world".split(" ").length
puts "hello".slice(1, 3)
puts "hello".starts_with?("hel")
puts "hello".ends_with?("llo")
puts "hello".include?("ell")
puts "  hello  ".strip
puts "HELLO".downcase
puts "hello".upcase
puts "hello".reverse

# === Array methods ===
arr = [3, 1, 2]
puts arr.sort.join(", ")
puts arr.reverse.join(", ")
puts arr.size
arr.unshift(0)
puts arr[0]
puts arr.shift

# === Array join without separator ===
nums = [1, 2, 3]
puts nums.join

# === Empty array shift ===
empty = []
puts empty.shift

# === Set methods ===
s = Set.new([1, 2, 3])
puts s.empty?
puts Set.new.empty?
puts s.to_a.length

# === Range methods ===
r = 1..5
total = 0
r.each do |n|
  total = total + n
end
puts total

# === Times iterator ===
count = 0
3.times do |i|
  count = count + 1
end
puts count

puts "error_paths_test passed"
