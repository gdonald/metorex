# Reflection and Introspection
# Demonstrates class, instance_of?, is_a?, methods, send, respond_to?

class Animal
  def speak
    "..."
  end
end

class Dog < Animal
  def speak
    "Woof!"
  end

  def fetch(item)
    "Fetching #{item}"
  end
end

puts "=== class ==="
d = Dog.new()
puts d.class
puts "hello".class
puts 42.class
arr = [1, 2]
puts arr.class

puts ""
puts "=== instance_of? ==="
puts d.instance_of?(Dog)
puts d.instance_of?(Animal)

puts ""
puts "=== is_a? ==="
puts d.is_a?(Dog)
puts d.is_a?(Animal)

puts ""
puts "=== respond_to? ==="
puts d.respond_to?("speak")
puts d.respond_to?("fetch")
puts d.respond_to?("nonexistent")

puts ""
puts "=== methods ==="
m = d.methods()
puts m.length

puts ""
puts "=== send ==="
puts d.send("speak")
puts d.send("fetch", "ball")

puts ""
puts "=== send with symbol ==="
puts d.send(:speak)
puts d.send(:fetch, "stick")

puts ""
puts "=== send on built-in ==="
puts "hello".send("upcase")
puts "hello".send("length")
