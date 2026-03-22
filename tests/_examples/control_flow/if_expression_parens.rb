# if/unless as expressions (with parentheses)
x = if true then 42 else 0 end
puts(x)

y = if false then 1 else 2 end
puts(y)

z = if nil then 1 end
puts(z)

w = unless false then 10 else 20 end
puts(w)

result = if 1 > 0
  "positive"
elsif 1 < 0
  "negative"
else
  "zero"
end
puts(result)
