module ASInternal
end

class ASParent
end

class ASChild < ASParent
  class << self
    include ASInternal
  end
end

sc = class << ASChild; self; end
puts sc.ancestors.inspect

puts "---"
puts sc.ancestors.include?(ASInternal)
puts sc.ancestors.include?(Class)
puts sc.ancestors.include?(Module)
puts sc.ancestors.include?(Object)
puts sc.ancestors.include?(Kernel)
