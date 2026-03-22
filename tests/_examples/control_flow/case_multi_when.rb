def classify(n)
  case n
  when 1, 2, 3 then "small"
  when 4, 5, 6 then "medium"
  when 7, 8, 9 then "large"
  else "other"
  end
end

puts classify(1)
puts classify(3)
puts classify(5)
puts classify(8)
puts classify(100)
