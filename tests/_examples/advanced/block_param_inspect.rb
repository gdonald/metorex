# Inspect what blk is when no block is passed
def inspect_blk(&blk)
  puts blk.class
  puts blk.nil?.to_s
end
inspect_blk
