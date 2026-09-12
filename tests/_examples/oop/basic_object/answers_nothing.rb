# An object rooted at BasicObject answers only what BasicObject defines.
# Everything Kernel and Object add arrives through Object, which such a class
# never inherits from.
held = BasicObject.new

begin
  held.to_s
rescue NoMethodError => refused
  puts(refused.message)
end

# A method taken from Kernel and bound onto one still runs.
p(::Kernel.instance_method(:frozen?).bind(held).call)
p(::Kernel.instance_method(:class).bind(held).call)

# A class rooted there may take Kernel on, and then it answers everything
# Kernel defines.
class WithKernel < BasicObject
  include ::Kernel
end

p(WithKernel.new.frozen?)

# A class may also take a single one of Kernel's methods on.
class OnlyRespondTo < BasicObject
  define_method(:respond_to?, ::Kernel.instance_method(:respond_to?))

  def named
    :named
  end
end

asked = OnlyRespondTo.new
p([asked.respond_to?(:named), asked.respond_to?(:missing)])
