# Pattern Matching Examples for Metorex
# Demonstrates various pattern matching features

# Example 1: Literal pattern matching
value = 42
match value
  when 0
    puts "Zero"
  when 42
    puts "The answer!"
  when _
    puts "Something else"
end

# Example 2: Identifier pattern (variable binding)
match 100
  when x
    puts "Matched: #{x}"
end

# Example 3: Array destructuring
arr = [1, 2, 3]
match arr
  when [a, b, c]
    puts "a=#{a}, b=#{b}, c=#{c}"
end

# Example 4: Array with rest pattern
numbers = [1, 2, 3, 4, 5]
match numbers
  when [first, ...rest]
    puts "First: #{first}"
    puts "Rest: #{rest}"
end

# Example 5: Rest in the middle
match numbers
  when [first, ...middle, last]
    puts "First: #{first}, Last: #{last}"
    puts "Middle: #{middle}"
end

# Example 6: Object/Dictionary destructuring
point = {x: 10, y: 20}
match point
  when {x, y}
    puts "Point at (#{x}, #{y})"
end

# Example 7: Guard clause
temperature = 75
match temperature
  when t if t > 80
    puts "Hot!"
  when t if t > 60
    puts "Warm"
  when t if t > 40
    puts "Cool"
  when _
    puts "Cold!"
end

# Example 8: Nested patterns
data = [[1, 2], [3, 4]]
match data
  when [[a, b], [c, d]]
    sum = a + b + c + d
    puts "Sum: #{sum}"
end

# Example 9: Mixed types in array
mixed = [1, "hello", true]
match mixed
  when [num, str, bool]
    puts "Number: #{num}"
    puts "String: #{str}"
    puts "Boolean: #{bool}"
end

# Example 10: Multiple cases with specific patterns
status = 404
match status
  when 200
    puts "OK"
  when 404
    puts "Not Found"
  when 500
    puts "Internal Server Error"
  when code
    puts "Status code: #{code}"
end

# Example 11: String pattern matching
command = "start"
match command
  when "start"
    puts "Starting..."
  when "stop"
    puts "Stopping..."
  when "restart"
    puts "Restarting..."
  when _
    puts "Unknown command"
end

# Example 12: Boolean pattern matching
flag = true
match flag
  when true
    puts "Flag is true"
  when false
    puts "Flag is false"
end

# Example 13: Nil pattern matching
maybe_value = nil
match maybe_value
  when nil
    puts "No value"
  when x
    puts "Value: #{x}"
end

# Example 14: Complex guard with multiple conditions
age = 25
match age
  when a if a >= 18 and a < 65
    puts "Working age"
  when a if a < 18
    puts "Minor"
  when _
    puts "Retirement age"
end

# Example 15: Wildcard in array patterns
data = [1, 2, 3, 4]
match data
  when [1, _, _, last]
    puts "First is 1, last is #{last}"
end
