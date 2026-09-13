# Array.try_convert and Hash.try_convert ask an object what it stands for, and
# answer nil where it stands for nothing. Array#sample draws without
# replacement, optionally from a generator the caller supplies.
p Array.try_convert [1, 2]
p Array.try_convert "nope"
p Hash.try_convert({ a: 1 })
p Hash.try_convert 7

stands_for_array = Object.new
def stands_for_array.to_ary
  [3, 4]
end
p Array.try_convert stands_for_array

wrong_shape = Object.new
def wrong_shape.to_hash
  :not_a_hash
end
begin
  Hash.try_convert(wrong_shape)
rescue TypeError => trouble
  p trouble.message
end

counting = Object.new
def counting.rand limit
  limit - 1
end

p [1, 2, 3, 4].sample(random: counting)
p [1, 2, 3, 4].sample(2, random: counting)
p [1, 2, 3].sample(10).sort
p [].sample

begin
  [1, 2].sample(-1)
rescue ArgumentError => trouble
  p trouble.message
end

refuses = Object.new
def refuses.rand limit
  limit
end
begin
  [1, 2].sample(random: refuses)
rescue RangeError => trouble
  p trouble.class
end
