# Object/Dictionary access in case/when statements
# Note: Standard Ruby doesn't support destructuring in case statements
# This demonstrates the pattern using regular hash access

# Working with point hash
point = {"x" => 10, "y" => 20}
case point
when Hash
  x = point["x"]
  y = point["y"]
  puts "Point at (#{x}, #{y})"
end

# Working with person hash
person = {"name" => "Alice", "age" => 30}
case person
when Hash
  n = person["name"]
  a = person["age"]
  puts "Name: #{n}, Age: #{a}"
end

# Another example
case person
when Hash
  name = person["name"]
  years = person["age"]
  puts "#{name} is #{years} years old"
end
