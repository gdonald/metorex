module M; end
class C; end
class D < C; end

# `Left === right` asks whether the right stands as one of the left, and a
# class is an object of Class just as an instance is an object of its class.
[[Module, String], [Class, String], [String, String], [Module, M], [Class, M],
 [C, D], [C, C], [Object, C], [Comparable, Integer], [Module, Comparable]].each do |left, right|
  puts "#{left} === #{right} -> #{left === right}"
end

puts (Module === M).inspect
puts M.is_a?(Module).inspect

# A program that writes its own Kernel function means that one.
module Kernel
  def p(*objects)
    STDOUT.puts "written here: #{objects.inspect}"
    objects.length == 1 ? objects.first : objects
  end
end
p 1
p 1, 2
