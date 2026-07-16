# Block parameter defaults — written with parentheses. `{ |a, b = 1| }`
# binds b to 1 when only one argument is supplied, both in direct block
# calls and in methods created via define_method. A trailing comma in the
# parameter list parses (`|a,|`).
pair = lambda { |a, b = 1| [a, b] }
puts(pair.call(5).inspect)
puts(pair.call(5, 6).inspect)

klass = Class.new do
  define_method(:m) { |a, b = 1| return a, b }
end
puts(klass.new.m(1).inspect)
puts(klass.new.m(1, 2).inspect)

begin
  klass.new.m
rescue ArgumentError
  puts("ArgumentError")
end

begin
  klass.new.m(1, 2, 3)
rescue ArgumentError
  puts("ArgumentError")
end

first_only = lambda { |a,| a }
puts(first_only.call(9).inspect)
