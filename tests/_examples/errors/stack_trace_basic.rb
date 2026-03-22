# Basic stack trace example

class Calculator
  def divide(x, y)
    self.validate(y)
    x / y
  end

  def validate(value)
    if value == 0
      raise("Division by zero!")
    end
  end
end

begin
  calc = Calculator.new
  calc.divide(10, 0)
rescue => e
  puts e.to_s
end
