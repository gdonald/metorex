module Basic
  def public_module; end
end

module Sup
  include Basic
  def public_super_module; end
end

puts Basic.ancestors.inspect
puts Sup.ancestors.inspect

puts "---"
puts Sup.ancestors == [Sup, Basic]
puts Sup.ancestors.first == Sup
puts Sup.ancestors[1] == Basic
puts "---"
puts [Sup, Basic].inspect
