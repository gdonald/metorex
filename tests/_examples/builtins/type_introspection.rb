# Type introspection demonstration

# is_a? checks
puts 42.is_a?(Integer)
puts 42.is_a?(String)
puts "hello".is_a?(String)
puts 3.14.is_a?(Float)
arr_check = [1, 2]
puts arr_check.is_a?(Array)

# kind_of? is an alias
puts 42.kind_of?(Integer)

# is_a? checks inheritance
puts 42.is_a?(Object)

# superclass
puts Integer.superclass.name
puts Object.superclass

# ancestors
puts Integer.ancestors.length

# Custom class introspection
class Animal
  def initialize(name)
    @name = name
  end
end

class Dog < Animal
  def initialize(name, breed)
    super(name)
    @breed = breed
  end
end

d = Dog.new("Rex", "Labrador")
puts d.is_a?(Dog)
puts d.is_a?(Animal)
puts Dog.superclass.name

# instance_variables
puts d.instance_variables.length

# instance_variable_get
puts d.instance_variable_get("@name")

# dup creates independent copy
arr = [1, 2, 3]
arr2 = arr.dup
arr2.push(4)
puts arr.length
puts arr2.length
