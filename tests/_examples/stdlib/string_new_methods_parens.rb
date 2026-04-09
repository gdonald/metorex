# to_i
puts "42".to_i
puts "  99  ".to_i
puts "hello".to_i
puts "-7".to_i
puts "3abc".to_i

# to_f
puts "3.14".to_f
puts "hello".to_f
puts "-2.5".to_f

# dup
s = "original"
d = s.dup
puts d

# gsub
puts "hello world".gsub("o", "0")
puts "aaa".gsub("a", "bb")

# sub (replace first only)
puts "hello hello".sub("hello", "hi")

# empty?
puts "".empty?
puts "hi".empty?
