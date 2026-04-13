# Test calling a method that accepts &blk without passing a block
def maybe_yield(&blk)
  if blk.nil?
    "no block"
  else
    blk.call(42)
  end
end
puts maybe_yield()
puts maybe_yield() { |x| x * 2 }
