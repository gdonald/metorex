# Basic case expression examples

# Simple literal matching with assignment
x = 2
result = case x
when 1 then "one"
when 2 then "two"
when 3 then "three"
else "other"
end
puts result

# Case expression as method argument
value = 42
puts(case value
when 42 then "The answer!"
when 0 then "Zero"
else "Unknown"
end)

# Case expression with no else returns nil
num = 99
output = case num
when 1 then "one"
when 2 then "two"
end
puts output

# Nested case expressions
a = 1
b = 2
result = case a
when 1 then case b
  when 2 then "a=1, b=2"
  else "a=1, b!=2"
  end
when 2 then "a=2"
else "other"
end
puts result
