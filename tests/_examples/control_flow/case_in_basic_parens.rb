# case/in pattern matching (Ruby 2.7+ style, with parentheses)
value = 42

case value
in Integer => n
  puts(n)
end

x = "hello"

case x
in String => s
  puts(s)
in Integer
  puts("number")
end

case [1, 2, 3]
in [Integer => a, Integer => b, Integer => c]
  puts(a)
  puts(b)
  puts(c)
end

case 99
in 99 => n
  puts(n)
end
