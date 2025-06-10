# Simple exception handling example

# Example 1: Basic rescue without exception type
begin
  puts "Before exception"
  raise "Something went wrong"
  puts "This should not print"
rescue
  puts "Caught an exception"
end

puts "After rescue block"

# Example 2: Rescue with exception binding
begin
  raise "An error message"
rescue => e
  puts "Caught exception with message: #{e}"
end

# Example 3: Ensure block always runs
begin
  puts "In try block"
  raise "Error"
rescue
  puts "In rescue block"
ensure
  puts "In ensure block"
end
