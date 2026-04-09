begin
  puts "try block"
  x = 42
rescue => e
  puts "error: " + e.to_s
else
  puts "no error, x = " + x.to_s
ensure
  puts "ensure ran"
end
