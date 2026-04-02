# method_missing without parentheses (where possible)

class DynamicRecord
  def initialize
    @attributes = {"name" => "Alice", "age" => 30, "role" => "engineer"}
  end

  def method_missing(name)
    if @attributes.has_key?(name)
      @attributes[name]
    else
      "unknown: #{name}"
    end
  end
end

puts "=== Dynamic Attribute Access ==="
record = DynamicRecord.new
puts record.name
puts record.age
puts record.role
puts record.email

class Ghost
  def method_missing(name, args)
    puts "Called #{name} with #{args.length} arg(s)"
  end
end

puts ""
puts "=== Ghost Methods ==="
ghost = Ghost.new
ghost.hello
ghost.add(1, 2)
ghost.greet("Alice", "Bob", "Charlie")

class FlexibleCalc
  def method_missing(name, args)
    if name == "sum"
      total = 0
      args.each do |n|
        total = total + n
      end
      total
    else
      "unknown operation: #{name}"
    end
  end
end

puts ""
puts "=== Flexible Calculator ==="
calc = FlexibleCalc.new
puts calc.sum(1, 2, 3)
puts calc.sum(10, 20)
puts calc.multiply(2, 3)

class Selective
  def real_method
    "I am real"
  end

  def method_missing(name)
    "ghost: #{name}"
  end
end

puts ""
puts "=== Selective ==="
s = Selective.new
puts s.real_method
puts s.fake_method

class Base
  def method_missing(name)
    "Base caught: #{name}"
  end
end

class Child < Base
end

puts ""
puts "=== Inherited method_missing ==="
c = Child.new
puts c.anything
puts c.whatever
