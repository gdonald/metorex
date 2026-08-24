class Vault
  @@shared = "password"

  def initialize(secret)
    @secret = secret
  end

  def square(n)
    n * n
  end

  def capture
    local = true
    binding
  end
end

context = Vault.new(99).capture

puts context.class
puts context.kind_of?(Binding)
puts eval("@secret", context)
puts eval("local", context)
puts eval("@@shared", context)
puts eval "square(2)", context
puts eval("self.square(2)", context)

eval "local = false", context
puts eval("local", context)

puts Kernel.binding.kind_of?(Binding)
puts Kernel.private_instance_methods.include?(:binding)
