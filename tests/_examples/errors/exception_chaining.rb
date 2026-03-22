# Exception Chaining Example
# Demonstrates exception cause chains and tracking the source of errors

# Example 1: Simple exception chain - catching and re-raising
puts "Example 1: Catching and re-raising"

class DatabaseError < StandardError
end

class NetworkError < StandardError
end

def connect_to_database
  raise NetworkError.new("Network connection failed")
end

def initialize_system
  begin
    connect_to_database()
  rescue NetworkError => e
    puts "Caught NetworkError: #{e.message}"
    puts "Re-raising as DatabaseError..."
    raise DatabaseError.new("Database initialization failed")
  end
end

begin
  initialize_system()
rescue DatabaseError => e
  puts "Caught DatabaseError: #{e.message}"
end

# Example 2: Multiple levels of exception handling
puts ""
puts "Example 2: Multi-level exception handling"

def level_1
  raise RuntimeError.new("Error at level 1")
end

def level_2
  begin
    level_1()
  rescue RuntimeError => e
    puts "Level 2 caught: #{e.message}"
    raise TypeError.new("Type error in level 2")
  end
end

def level_3
  begin
    level_2()
  rescue TypeError => e
    puts "Level 3 caught: #{e.message}"
    raise ValueError.new("Value error in level 3")
  end
end

begin
  level_3()
rescue ValueError => e
  puts "Top level caught: #{e.message}"
end

# Example 3: Using $! to access current exception
puts ""
puts "Example 3: Accessing current exception with $!"

begin
  raise RuntimeError.new("Original error")
rescue => e
  puts "Caught exception: #{e.message}"
  puts "Exception binding and $! both reference the current exception"
end

# Example 4: Preserving error context through layers
puts ""
puts "Example 4: Error context preservation"

class ConfigError < StandardError
end

class FileError < StandardError
end

def read_config_file
  raise FileError.new("config.txt not found")
end

def load_configuration
  begin
    read_config_file()
  rescue FileError => e
    puts "File error occurred: #{e.message}"
    raise ConfigError.new("Failed to load configuration")
  end
end

def start_application
  begin
    load_configuration()
  rescue ConfigError => e
    puts "Configuration error: #{e.message}"
    puts "Application cannot start"
  end
end

start_application()

# Example 5: Conditional re-raising
puts ""
puts "Example 5: Conditional re-raising"

def risky_operation(should_recover)
  begin
    raise RuntimeError.new("Something went wrong")
  rescue RuntimeError => e
    if should_recover
      puts "Recovered from error: #{e.message}"
    else
      puts "Cannot recover, re-raising..."
      raise
    end
  end
end

risky_operation(true)

begin
  risky_operation(false)
rescue RuntimeError => e
  puts "Caught re-raised error: #{e.message}"
end
