# A class descending from BasicObject never reaches top-level constants,
# because Ruby finds those among Object's own. A leading `::` still does.

class Bare < BasicObject
  def self.sees_kernel
    defined?(Kernel)
  end

  def self.sees_kernel_at_top_level
    ::Kernel.name
  end

  include ::Kernel
end

puts(Bare.sees_kernel.inspect)
puts(Bare.sees_kernel_at_top_level)

begin
  class Bare
    Kernel
  end
rescue NameError => error
  puts(error.message)
end

instance = Bare.new
instance.instance_variable_set(:@stored, :value)
puts(instance.instance_variable_get(:@stored).inspect)
puts(instance.respond_to?(:hash).to_s)

puts(BasicObject.const_defined?(:Kernel).to_s)
puts(BasicObject.constants(false).inspect)
puts(BasicObject::BasicObject.name)
puts(Object.constants(false).include?(:BasicObject).to_s)

metaclass = class << BasicObject; self; end
puts(metaclass.instance_of?(Class).to_s)
puts(metaclass.superclass.name)

::TopLevelBinding = Class.new
puts(TopLevelBinding.name)
