# A for loop that skips one value with next
for i in 1..5
  if i == 3
    next
  end
  puts i
end
