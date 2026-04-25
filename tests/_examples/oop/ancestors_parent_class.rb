class AParent
end

puts AParent.ancestors.inspect
puts [AParent, Object, Kernel, BasicObject].inspect
puts AParent.ancestors == [AParent, Object, Kernel, BasicObject]
puts "---"
puts AParent.ancestors.first == AParent
puts AParent.ancestors[1] == Object
puts AParent.ancestors[2] == Kernel
puts AParent.ancestors[3] == BasicObject
