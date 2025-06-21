# Basic Error Handling Examples for Metorex
#
# This file demonstrates the error handling capabilities of Metorex,
# including syntax errors, runtime errors, and type errors.
# Note: This is a demonstration file showing how errors would be reported.

# Example 1: Syntax Error
# This would cause a syntax error due to missing 'end' keyword
def broken_function
  puts "This function has no end"
# Missing 'end' here - SyntaxError at line 11:1

# Example 2: Runtime Error
# Division by zero would cause a runtime error
def divide(a, b)
  result = a / b  # RuntimeError if b is 0
  return result
end

# This call would trigger a RuntimeError
# divide(10, 0)

# Example 3: Type Error
# Type mismatch in operations
def add_numbers(x, y)
  return x + y
end

# This would cause a TypeError if Metorex has type checking
# result = add_numbers(5, "hello")  # TypeError: Cannot add Int and String

# Example 4: Error with Stack Trace
def level3
  divide(10, 0)  # This will error
end

def level2
  level3()  # Called from level2
end

def level1
  level2()  # Called from level1
end

# Calling level1 would produce a stack trace:
# RuntimeError at examples/errors/basic_error.mx:33:5: Division by zero
#   at level3 (examples/errors/basic_error.mx:33:5)
#   at level2 (examples/errors/basic_error.mx:37:5)
#   at level1 (examples/errors/basic_error.mx:41:5)

# Example 5: Proper Error Handling (when implemented)
def safe_divide(a, b)
  begin
    result = a / b
    return result
  rescue DivisionByZeroError => e
    puts "Error: Cannot divide by zero"
    return nil
  end
end

# Example 6: Type Error with Expected vs Found
def process_user(user: User)
  puts user.name
end

# This would produce: TypeError: Expected User, found String
# process_user("not a user")

# Example 7: Multiple Error Types
class Calculator
  def compute(operation, x, y)
    if operation == "divide"
      if y == 0
        raise RuntimeError.new("Cannot divide by zero")
      end
      return x / y
    elsif operation == "add"
      return x + y
    else
      raise RuntimeError.new("Unknown operation: #{operation}")
    end
  end
end

# Error reporting features demonstrated:
# 1. Source location (file:line:column)
# 2. Error message with context
# 3. Source code snippet with error indicator (^)
# 4. Stack traces for runtime errors
# 5. Expected vs found types for type errors
