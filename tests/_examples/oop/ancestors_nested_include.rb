module NIBasic
  def public_module; end
end

module NISuper
  include NIBasic
  def public_super_module; end
end

class NIParent
  def parent_method; end
end

class NIChild < NIParent
  include NISuper
end

puts NIBasic.ancestors.inspect
puts NISuper.ancestors.inspect
puts NIParent.ancestors.inspect
puts NIChild.ancestors.inspect
puts "---"
puts NIChild.ancestors == [NIChild, NISuper, NIBasic, NIParent, Object, Kernel, BasicObject]
