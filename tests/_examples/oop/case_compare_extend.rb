module Basic
end

module Super
  include Basic
end

class Plain
end

obj = Plain.new
obj.extend Super

puts "Basic === obj: #{Basic === obj}"
puts "Super === obj: #{Super === obj}"
puts "obj.is_a?(Basic): #{obj.is_a? Basic}"
puts "obj.is_a?(Super): #{obj.is_a? Super}"

class Parent
end

class Child < Parent
  include Super
end

puts "Basic === Child.new: #{Basic === Child.new}"
puts "Super === Child.new: #{Super === Child.new}"
