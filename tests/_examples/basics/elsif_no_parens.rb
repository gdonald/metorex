# elsif without parentheses in conditions
temperature = 25

if temperature < 0
  puts "freezing"
elsif temperature < 15
  puts "cold"
elsif temperature < 25
  puts "moderate"
elsif temperature < 35
  puts "warm"
else
  puts "hot"
end
