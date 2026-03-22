# Array destructuring in case/when statements

# Simple array destructuring
arr = [1, 2, 3]
case arr
  when [a, b, c]
    puts "a=#{a}, b=#{b}, c=#{c}"
end

# Array with rest pattern
numbers = [1, 2, 3, 4, 5]
case numbers
  when [first, ...rest]
    puts "First: #{first}"
    puts "Rest: #{rest}"
end

# Rest in the middle
case numbers
  when [first, ...middle, last]
    puts "First: #{first}, Last: #{last}"
    puts "Middle: #{middle}"
end

# Nested array patterns
data = [[1, 2], [3, 4]]
case data
  when [[a, b], [c, d]]
    sum = a + b + c + d
    puts "Sum: #{sum}"
end

# Wildcard in array patterns
data2 = [1, 2, 3, 4]
case data2
  when [1, _, _, last]
    puts "First is 1, last is #{last}"
end
