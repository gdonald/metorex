# Case expression with mixed inline/block syntax

# String pattern matching with inline then
name = "Alice"
greeting = case name
when "Alice" then "Hello, Alice!"
when "Bob" then "Hi, Bob!"
else "Hello, stranger!"
end
puts greeting

# Numeric matching with mixed syntax
value = 2
msg = case value
when 1 then "one"
when 2
  "two"
when 3 then "three"
else
  "other"
end
puts msg

# Type pattern matching
score = 85
grade = case score
when Integer then "B"
else "F"
end
puts grade

# Wildcard pattern
day = 6
day_type = case day
when 6 then "weekend"
when _ then "other"
end
puts day_type
