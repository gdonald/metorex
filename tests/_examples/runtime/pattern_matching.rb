# Pattern matching with `case` and `in`: literals, bindings, array and hash
# patterns, rest patterns, guards and nesting.

# Example 1: Literal pattern matching
value = 42
case value
in 0
  puts "Zero"
in 42
  puts "The answer!"
in _
  puts "Something else"
end

# Example 2: Identifier pattern (variable binding)
case 100
in x
  puts "Matched: #{x}"
end

# Example 3: Array destructuring
arr = [1, 2, 3]
case arr
in [a, b, c]
  puts "a=#{a}, b=#{b}, c=#{c}"
end

# Example 4: Array with rest pattern
numbers = [1, 2, 3, 4, 5]
case numbers
in [first, *rest]
  puts "First: #{first}"
  puts "Rest: #{rest}"
end

# Example 5: Rest in the middle
case numbers
in [first, *middle, last]
  puts "First: #{first}, Last: #{last}"
  puts "Middle: #{middle}"
end

# Example 6: Object/Dictionary destructuring
point = {x: 10, y: 20}
case point
in {x:, y:}
  puts "Point at (#{x}, #{y})"
end

# Example 7: Guard clause
temperature = 75
case temperature
in t if t > 80
  puts "Hot!"
in t if t > 60
  puts "Warm"
in t if t > 40
  puts "Cool"
in _
  puts "Cold!"
end

# Example 8: Nested patterns
data = [[1, 2], [3, 4]]
case data
in [[a, b], [c, d]]
  sum = a + b + c + d
  puts "Sum: #{sum}"
end

# Example 9: Mixed types in array
mixed = [1, "hello", true]
case mixed
in [num, str, bool]
  puts "Number: #{num}"
  puts "String: #{str}"
  puts "Boolean: #{bool}"
end

# Example 10: Multiple cases with specific patterns
status = 404
case status
in 200
  puts "OK"
in 404
  puts "Not Found"
in 500
  puts "Internal Server Error"
in code
  puts "Status code: #{code}"
end

# Example 11: String pattern matching
command = "start"
case command
in "start"
  puts "Starting..."
in "stop"
  puts "Stopping..."
in "restart"
  puts "Restarting..."
in _
  puts "Unknown command"
end

# Example 12: Boolean pattern matching
flag = true
case flag
in true
  puts "Flag is true"
in false
  puts "Flag is false"
end

# Example 13: Nil pattern matching
maybe_value = nil
case maybe_value
in nil
  puts "No value"
in x
  puts "Value: #{x}"
end

# Example 14: Complex guard with multiple conditions
age = 25
case age
in a if a >= 18 and a < 65
  puts "Working age"
in a if a < 18
  puts "Minor"
in _
  puts "Retirement age"
end

# Example 15: Wildcard in array patterns
data = [1, 2, 3, 4]
case data
in [1, _, _, last]
  puts "First is 1, last is #{last}"
end
