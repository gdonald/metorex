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
  # `to_s` is the message alone; the class and the backtrace are their own
  # methods.
  puts e.to_s
  puts e.class.to_s
  puts e.message
  puts e.backtrace.class.to_s
end
