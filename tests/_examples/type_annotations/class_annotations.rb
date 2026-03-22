class Person
  def initialize(name, age)  # str, int
    @name = name
    @age = age
  end

  def get_info  # -> str
    @name
  end
end

class Calculator
  def add(a, b)  # int, int -> int
    a + b
  end

  def multiply(x, y)  # float, float -> float
    x * y
  end
end

person = Person.new("Alice", 30)  # Person
puts person.get_info

calc = Calculator.new  # Calculator
result = calc.add(5, 10)
puts "5 + 10 = #{result}"

product = calc.multiply(2.5, 4.0)
puts "2.5 * 4.0 = #{product}"
