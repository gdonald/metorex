# Error location tracking example

def dangerous_operation(x)
  if x < 0
    raise("Negative value not allowed")
  end
  x * 2
end

def process(value)
  dangerous_operation(value)
end

begin
  process(-5)
rescue => e
  puts "Error: #{e.message}"
  puts "Type: #{e.type}"
end
