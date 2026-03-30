# Calculator: A class-based calculator with basic arithmetic operations

class Calculator
  def initialize
    @result = 0
  end

  def result
    @result
  end

  def add(a, b)
    @result = a + b
    @result
  end

  def subtract(a, b)
    @result = a - b
    @result
  end

  def multiply(a, b)
    @result = a * b
    @result
  end

  def divide(a, b)
    if b == 0
      puts "Error: Division by zero"
      return nil
    end
    @result = a / b
    @result
  end

  def modulo(a, b)
    if b == 0
      puts "Error: Modulo by zero"
      return nil
    end
    @result = a % b
    @result
  end
end

calc = Calculator.new

puts "Addition: 10 + 3 = #{calc.add(10, 3)}"
puts "Subtraction: 10 - 3 = #{calc.subtract(10, 3)}"
puts "Multiplication: 10 * 3 = #{calc.multiply(10, 3)}"
puts "Division: 10 / 3 = #{calc.divide(10, 3)}"
puts "Modulo: 10 % 3 = #{calc.modulo(10, 3)}"
puts "Division by zero:"
calc.divide(10, 0)
puts "Last result: #{calc.result}"
