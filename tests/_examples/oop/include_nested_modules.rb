module Leaf
  LABEL = 'leaf'

  def leaf_name
    :leaf
  end
end

module Branch
end

module Trunk
  LABEL = 'trunk'

  def self.label
    LABEL
  end
end

class Tree
  include Branch
end

class Sapling
  include Leaf
end

class Seedling < Sapling
  include Leaf
end

puts Trunk.label.inspect
Trunk.include Leaf
puts Trunk.label.inspect

Branch.include Leaf
puts Tree.new.leaf_name.inspect
puts Tree.instance_methods.inspect
puts Seedling.ancestors[0, 3].inspect
