# Error Reporting and Stack Traces Examples
# Demonstrates how Metorex reports errors with source locations and stack traces

# Example 1: Basic runtime error
# Uncommenting this will show an error with line number
# undefined_variable

# Example 2: Division by zero error
begin
  x = 10
  y = 0
  result = x / y  # This will error with location
rescue => e
  puts "Caught error: #{e}"
end

# Example 3: Type error
begin
  result = "hello" + 42  # Cannot add string and integer
rescue => e
  puts "Type error caught: #{e}"
end

# Example 4: Array index out of bounds
begin
  arr = [1, 2, 3]
  value = arr[100]  # Index out of bounds
rescue => e
  puts "Index error: #{e}"
end

# Example 5: Undefined method
begin
  x = 42
  x.undefined_method  # Method doesn't exist on integers
rescue => e
  puts "Method error: #{e}"
end

# Example 6: Stack trace demonstration
# This shows nested method calls in the error trace

class Calculator
  def calculate(x, y)
    validate(x, y)
    perform_operation(x, y)
  end

  def validate(x, y)
    check_values(x, y)
  end

  def check_values(x, y)
    if y == 0
      raise "Cannot divide by zero!"
    end
  end

  def perform_operation(x, y)
    x / y
  end
end

begin
  calc = Calculator.new
  result = calc.calculate(10, 0)
rescue => e
  puts "Error with stack trace: #{e}"
  # The stack trace will show:
  # - check_values (where the error occurred)
  # - validate (which called check_values)
  # - calculate (which called validate)
end

# Example 7: Pattern matching error
begin
  match 5
    when 1
      puts "one"
    when 2
      puts "two"
    # No pattern matches 5, so this will error
  end
rescue => e
  puts "Pattern match error: #{e}"
end

# Example 8: Break/continue outside loop
begin
  break  # Error: break outside of loop
rescue => e
  puts "Control flow error: #{e}"
end

# Example 9: Self outside method context
begin
  x = self  # Error: self undefined outside method
rescue => e
  puts "Context error: #{e}"
end

# Example 10: Deeply nested error with full stack trace
class Deep
  def level1
    puts "Entering level 1"
    level2
  end

  def level2
    puts "Entering level 2"
    level3
  end

  def level3
    puts "Entering level 3"
    level4
  end

  def level4
    puts "Entering level 4"
    level5
  end

  def level5
    puts "Entering level 5"
    # This will cause an error
    undefined_variable
  end
end

begin
  obj = Deep.new
  obj.level1
rescue => e
  puts "Deep error caught: #{e}"
  # Stack trace will show all five levels:
  # level5 -> level4 -> level3 -> level2 -> level1
end

# Example 11: Method argument mismatch
class Greeter
  def greet(name, title)
    puts "Hello, #{title} #{name}"
  end
end

begin
  greeter = Greeter.new
  greeter.greet("Alice")  # Missing 'title' argument
rescue => e
  puts "Argument error: #{e}"
end

# Example 12: Invalid assignment target
begin
  42 = 10  # Cannot assign to a literal
rescue => e
  puts "Assignment error: #{e}"
end

# Example 13: Calling non-callable object
begin
  x = 42
  x()  # Integers are not callable
rescue => e
  puts "Callable error: #{e}"
end

# Example 14: Exception in guard clause
begin
  match 10
    when x if x > undefined_var  # Error in guard
      puts "matched"
  end
rescue => e
  puts "Guard error: #{e}"
end

# Example 15: Custom exception with raise
class ValidationError
end

begin
  raise ValidationError
rescue ValidationError => e
  puts "Custom exception caught"
end

puts "Error reporting examples completed!"
