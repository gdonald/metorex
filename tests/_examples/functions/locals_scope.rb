total = -1
doubles = [1, 2, 3]

def stats(n)
  total = 0
  doubles = (0...n).map { |i| i * 2 }
  doubles.each do |value|
    total = total + value
  end
  puts total
  puts doubles
end

stats(5)
