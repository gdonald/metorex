module MSpecMatchers
  private def equal(expected)
    "EqualMatcher:#{expected}"
  end
end

class MSpecEnv
  include MSpecMatchers
end

ENV_OBJ = MSpecEnv.new

def store_block(&block)
  block
end

def protect_it(blocks)
  arr = Array(blocks)
  arr.all? do |blk|
    ENV_OBJ.instance_exec(&blk)
    true
  end
end

$result = nil
blk = store_block { $result = equal(42) }
protect_it(blk)
puts $result
