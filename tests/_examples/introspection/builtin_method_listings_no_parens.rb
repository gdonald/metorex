# The method listings of a built-in value or class name the methods Ruby's
# core gives it, by visibility, alongside the ones a program wrote.
class NilClass
  def blank? = true
end

p [1].methods.include?(:first)
p ([1].methods - Object.instance_methods).include?(:each_slice)
p "text".public_methods.include?(:upcase)
p 1.private_methods.include?(:Integer)
p 1.private_methods.include?(:puts)
p Object.new.methods.include?(:puts)
p nil.methods.include?(:blank?)
p NilClass.instance_methods(false).sort
p Comparable.instance_methods.sort
p String.instance_methods(false).include?(:upcase)
p Hash.public_instance_methods.include?(:fetch)
p Kernel.private_instance_methods.include?(:iterator?)
p Proc.new {}.private_methods.include?(:binding)
p RuntimeError.new("m").methods.include?(:initialize)
p RuntimeError.new("m").private_methods.include?(:initialize)
p File.instance_methods.include?(:fdatasync)
p IO.instance_methods.include?(:to_path)
p TOPLEVEL_BINDING.receiver.private_methods.include?(:include)
p Object.new.respond_to?(:initialize, true)
p Object.new.respond_to?(:Integer, true)
p [1].frist rescue p $!.corrections
p :sym.to_proce rescue p $!.corrections
