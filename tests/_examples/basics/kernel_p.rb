p "abcde"
p 42
p :symbol
p nil
p [1, :two, "three"]

class Widget
  def inspect
    "custom inspect"
  end
end

p Widget.new

one = p 7
puts one.inspect

several = p 1, 2
puts several.inspect

none = p
puts none.inspect

puts Kernel.private_instance_methods(false).include?(:p)
