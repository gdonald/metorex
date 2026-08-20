puts Integer "42"
puts Integer 3.9
puts String 42
puts String nil
puts Array nil
pair = [1, 2]
puts Array pair
puts Array "hi"

begin
  Integer true
rescue TypeError => e
  puts e.message
end

begin
  Integer nil
rescue TypeError => e
  puts e.message
end
