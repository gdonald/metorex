# Case expression with various pattern types

# Array destructuring patterns
arr = [1, 2, 3]
result = case arr
when [a, b, c] then a + b + c
else 0
end
puts result

# Array destructuring with rest pattern
data = [10, 20, 30, 40]
sum = case data
when [first, ...rest] then first
else 0
end
puts sum

# Variable binding pattern
value = 42
bound = case value
when x then x * 2
end
puts bound

# Type pattern matching
obj = 123
type_result = case obj
when Integer then "It's an integer"
when String then "It's a string"
else "Unknown type"
end
puts type_result

# Wildcard pattern
anything = "test"
wild = case anything
when _ then "matches anything"
end
puts wild

# Hash/Dictionary destructuring
dict = {"x" => 10, "y" => 20}
point_sum = case dict
when {x: a, y: b} then a + b
else 0
end
puts point_sum
