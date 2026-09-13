# Array#to_h names the index of the element it refused, and it walks the array
# as it stands rather than a snapshot taken up front.
p([[1, 2], [3, 4]].to_h)
p([[1, 2]].to_h { |key, value| [key.to_s, value * 2] })

begin
  [[1, 2, 3]].to_h
rescue ArgumentError => trouble
  p trouble.message
end

begin
  ["pair"].to_h
rescue TypeError => trouble
  p trouble.message
end

class Pairable
  def to_ary
    [:key, :value]
  end
end

p([Pairable.new].to_h)

growing = [[1, 1]]
seen = []
growing.to_h do |pair|
  seen << pair
  growing << [2, 2] if growing.size == 1
  pair
end
p seen
