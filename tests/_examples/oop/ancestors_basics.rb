puts BasicObject.ancestors.inspect
puts Object.ancestors.inspect
puts Kernel.ancestors.inspect

module MSpecsAncestors
end
puts MSpecsAncestors.ancestors.inspect

class MSABasic
end
puts MSABasic.ancestors.inspect

class MSASuper < MSABasic
end
puts MSASuper.ancestors.inspect

class MSAParent
end
class MSAChild < MSAParent
end
puts MSAParent.ancestors.inspect
puts MSAChild.ancestors.inspect
