module ASMStandalone
end

puts ASMStandalone.singleton_class.ancestors.inspect
puts ASMStandalone.singleton_class.ancestors.include?(Module)
puts ASMStandalone.singleton_class.ancestors.include?(Object)
puts ASMStandalone.singleton_class.ancestors.include?(Kernel)
puts ASMStandalone.singleton_class.ancestors.include?(BasicObject)
