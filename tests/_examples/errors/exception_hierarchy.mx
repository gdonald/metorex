# Exception Hierarchy Example
# Demonstrates the standard exception types and their inheritance

# Example 1: Catching different exception types
puts "Example 1: Different exception types"

begin
  raise RuntimeError.new("Runtime error occurred")
rescue RuntimeError => e
  puts "Caught RuntimeError: #{e.message}"
end

begin
  raise TypeError.new("Type mismatch")
rescue TypeError => e
  puts "Caught TypeError: #{e.message}"
end

begin
  raise ValueError.new("Invalid value")
rescue ValueError => e
  puts "Caught ValueError: #{e.message}"
end

# Example 2: Catching base StandardError
puts ""
puts "Example 2: Catching StandardError"

begin
  raise RuntimeError.new("A runtime error")
rescue StandardError => e
  puts "Caught as StandardError: #{e.message}"
end

begin
  raise TypeError.new("A type error")
rescue StandardError => e
  puts "Caught as StandardError: #{e.message}"
end

# Example 3: Exception hierarchy - more specific catches first
puts ""
puts "Example 3: Specific to general exception handling"

def handle_error(error_type)
  begin
    if error_type == "runtime"
      raise RuntimeError.new("Runtime issue")
    elsif error_type == "type"
      raise TypeError.new("Type issue")
    elsif error_type == "value"
      raise ValueError.new("Value issue")
    end
  rescue RuntimeError => e
    puts "Specific handler for RuntimeError: #{e.message}"
  rescue TypeError => e
    puts "Specific handler for TypeError: #{e.message}"
  rescue StandardError => e
    puts "General handler for StandardError: #{e.message}"
  end
end

handle_error("runtime")
handle_error("type")
handle_error("value")

# Example 4: Demonstrating exception inheritance
puts ""
puts "Example 4: Exception type checking"

begin
  raise RuntimeError.new("Test error")
rescue StandardError => e
  puts "RuntimeError is a StandardError: true"
  puts "Error message: #{e.message}"
end
