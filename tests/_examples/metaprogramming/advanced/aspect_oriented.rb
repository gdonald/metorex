# Aspect-Oriented Programming: Cross-cutting concerns via wrapping
# Demonstrates: blocks, closures, dynamic method definition

# A simple "before/after" aspect wrapper using blocks
def with_logging(name, &block)
  puts ">> entering #{name}"
  result = block.call
  puts "<< exiting #{name} => #{result}"
  result
end

puts "=== Before/After Aspects ==="
with_logging("calculation") do
  2 + 3
end

with_logging("string op") do
  "hello".upcase
end

# Timing aspect
def with_timing(name, &block)
  puts "Starting #{name}..."
  result = block.call
  puts "Finished #{name}"
  result
end

puts ""
puts "=== Timing Aspect ==="
with_timing("data processing") do
  total = 0
  for i in 1..100
    total += i
  end
  total
end

# Retry aspect
def with_retry(attempts, &block)
  tries = 0
  success = false
  final_result = nil
  while tries < attempts && !success
    tries += 1
    puts "Attempt #{tries}..."
    result = block.call(tries)
    if result
      puts "Success on attempt #{tries}"
      success = true
      final_result = result
    end
  end
  if !success
    puts "All #{attempts} attempts failed"
  end
  final_result
end

puts ""
puts "=== Retry Aspect ==="
with_retry(3) do |attempt|
  r = false
  if attempt >= 2
    r = "got it"
  end
  r
end
