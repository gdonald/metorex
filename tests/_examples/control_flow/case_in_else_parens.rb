# case/in with else clause (no NoMatchingPatternError), with parentheses
value = "not a number"

case value
in Integer => n
  puts(n)
else
  puts("no match")
end

case nil
in Integer
  puts("integer")
in String
  puts("string")
else
  puts("nil or other")
end
