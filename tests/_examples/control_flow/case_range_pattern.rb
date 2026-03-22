# Range patterns in case/when
score = 85

case score
when 90..100
  puts "A"
when 80..89
  puts "B"
when 70..79
  puts "C"
when 60..69
  puts "D"
else
  puts "F"
end

x = 5

case x
when 1..3
  puts "low"
when 4...7
  puts "mid"
when 7..10
  puts "high"
end

y = 7

case y
when 1...7
  puts "exclusive"
else
  puts "inclusive or out"
end
