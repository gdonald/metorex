puts 1 / 0.0
puts(-1 / 0.0)
puts 0 / 0.0
puts 1.0 / 0
puts 1.0 / 0.0
puts 4.0 / 2

begin
  1 / 0
rescue ZeroDivisionError, RuntimeError => e
  puts "integer division raises"
end
