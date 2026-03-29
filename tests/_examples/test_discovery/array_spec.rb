# Test file: array_spec.rb
# Discovered by --test flag (matches *_spec.rb pattern)

arr = [1, 2, 3]
assert_equal(3, arr.length)
assert_equal(1, arr[0])
assert_equal(3, arr[2])

arr.push(4)
assert_equal(4, arr.length)

puts "array_spec passed"
