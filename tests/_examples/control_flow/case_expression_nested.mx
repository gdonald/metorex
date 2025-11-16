# Nested case expressions and complex contexts

# Case expression in arithmetic
x = 2
computed = 10 + case x
when 1 then 5
when 2 then 15
else 0
end
puts computed

# Deeply nested case expressions
a = 1
b = 2
c = 3
nested_result = case a
when 1 then case b
  when 2 then case c
    when 3 then "all match"
    else "c no match"
    end
  else "b no match"
  end
else "a no match"
end
puts nested_result

# Case expression in array literal
val = 2
arr = [1, case val when 2 then 20 else 0 end, 3]
puts arr[0]
puts arr[1]
puts arr[2]

# Case expression used multiple times in one expression
x = 1
y = 2
sum = (case x when 1 then 10 else 0 end) + (case y when 2 then 20 else 0 end)
puts sum

# Nested case expressions with different match values
category = "food"
item = "apple"
result = case category
when "food" then case item
  when "apple" then "fruit"
  when "carrot" then "vegetable"
  else "unknown food"
  end
when "color" then case item
  when "red" then "warm color"
  when "blue" then "cool color"
  else "unknown color"
  end
else "unknown category"
end
puts result

# Nested case with outer scope reference
base = 100
modifier = 2
value = 50
computed = case value
when 50 then base + (case modifier
  when 1 then 10
  when 2 then 20
  when 3 then 30
  else 0
  end)
when 100 then base * 2
else base
end
puts computed

# Multiple nested cases in single expression
x = 1
y = 2
combined = (case x when 1 then 100 when 2 then 200 else 0 end) + (case y when 2 then 50 when 3 then 75 else 0 end)
puts combined

# Nested case in else clause
status = "unknown"
fallback = case status
when "active" then "running"
when "paused" then "waiting"
else case status
  when "unknown" then "undefined state"
  when "error" then "failed state"
  else "invalid"
  end
end
puts fallback
