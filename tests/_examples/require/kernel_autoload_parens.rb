target = File.expand_path("autoload_lib/kernel_autoload_target.rb", __dir__)

autoload(:AutoloadTarget, target)

puts(autoload?(:AutoloadTarget) == target)
puts(Object.autoload?(:AutoloadTarget) == target)
puts(autoload?(:NeverRegistered).inspect)

def registered(name)
  autoload?(name)
end

puts(registered(:AutoloadTarget) == target)

puts(AutoloadTarget.loaded)
puts(autoload?(:AutoloadTarget).inspect)
puts(Kernel.private_instance_methods.include?(:autoload))
puts(Kernel.private_instance_methods.include?(:autoload?))
