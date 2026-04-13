# Test passing a block as an argument using & syntax
def apply(&blk)
  blk.call(5)
end

b = lambda { |x| x * 3 }
puts apply(&b)
