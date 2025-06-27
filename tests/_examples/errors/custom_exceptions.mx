# Custom Exception Classes Example
# Demonstrates how to create and use custom exception types

# Example 1: Define custom exception classes
class DatabaseError < StandardError
end

class ConnectionError < DatabaseError
end

class QueryError < DatabaseError
end

class ValidationError < StandardError
end

# Example 2: Raising and catching custom exceptions
puts "Example 1: Custom exception types"

begin
  raise DatabaseError.new("Database connection failed")
rescue DatabaseError => e
  puts "Caught DatabaseError: #{e.message}"
end

begin
  raise ConnectionError.new("Could not connect to database")
rescue ConnectionError => e
  puts "Caught ConnectionError: #{e.message}"
end

begin
  raise QueryError.new("Invalid SQL query")
rescue QueryError => e
  puts "Caught QueryError: #{e.message}"
end

# Example 3: Catching parent exception class
puts ""
puts "Example 2: Catching via parent class"

begin
  raise ConnectionError.new("Connection timeout")
rescue DatabaseError => e
  puts "Caught as DatabaseError: #{e.message}"
end

begin
  raise QueryError.new("Table not found")
rescue DatabaseError => e
  puts "Caught as DatabaseError: #{e.message}"
end

# Example 4: Multiple rescue clauses with custom exceptions
puts ""
puts "Example 3: Multiple rescue clauses"

def process_database_operation(operation_type)
  begin
    if operation_type == "connect"
      raise ConnectionError.new("Connection failed")
    elsif operation_type == "query"
      raise QueryError.new("Query syntax error")
    elsif operation_type == "validate"
      raise ValidationError.new("Invalid input data")
    end
  rescue ConnectionError => e
    puts "Connection issue: #{e.message}"
  rescue QueryError => e
    puts "Query issue: #{e.message}"
  rescue ValidationError => e
    puts "Validation issue: #{e.message}"
  rescue DatabaseError => e
    puts "General database issue: #{e.message}"
  rescue StandardError => e
    puts "General error: #{e.message}"
  end
end

process_database_operation("connect")
process_database_operation("query")
process_database_operation("validate")

# Example 5: Re-raising custom exceptions
puts ""
puts "Example 4: Re-raising exceptions"

def attempt_operation
  begin
    raise QueryError.new("Failed to execute query")
  rescue QueryError => e
    puts "Caught in attempt_operation: #{e.message}"
    raise
  end
end

begin
  attempt_operation()
rescue QueryError => e
  puts "Caught in outer scope: #{e.message}"
end

# Example 6: Custom exception hierarchy usage
puts ""
puts "Example 5: Exception hierarchy in action"

def handle_database_work
  begin
    raise ConnectionError.new("Database unreachable")
  rescue ConnectionError => e
    puts "Specific handler: #{e.message}"
  rescue DatabaseError => e
    puts "General handler: #{e.message}"
  end
end

handle_database_work()
