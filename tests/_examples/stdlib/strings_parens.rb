s = "hello world"
puts(s.length)
puts(s.size)
puts(s.upcase)
puts(s.downcase)
puts(s.reverse)

puts("  hello  ".strip)
puts("  hello  ".trim)

parts = s.split(" ")
puts(parts.length)
puts(parts[0])
puts(parts[1])

words = "one,two,three".split(",")
puts(words.join(", "))
puts(words.join)

puts(s.slice(0, 5))
puts(s.slice(6, 5))
puts(s.slice(-5, 5))

puts(s.include?("hello"))
puts(s.include?("xyz"))
puts(s.contains?("world"))

puts(s.starts_with?("hello"))
puts(s.starts_with?("world"))
puts(s.ends_with?("world"))
puts(s.ends_with?("hello"))
