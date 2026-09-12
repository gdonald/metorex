# `autoload` inside a module's method registers on that module, since that is
# the scope the call sits in. `Kernel.autoload` registers in the same place.
target = File.expand_path("autoload_lib/module_autoload_target.rb", __dir__)

module Holder
  def register(file)
    autoload(:FromModuleMethod, file)
  end

  def register_through_kernel(file)
    Kernel.autoload(:AlsoFromModuleMethod, file)
  end
end

class Uses
  include Holder
end

Uses.new.register(target)
puts(Holder.autoload?(:FromModuleMethod) == target)
puts(Uses.autoload?(:FromModuleMethod) == target)
puts(Object.autoload?(:FromModuleMethod).inspect)

Uses.new.register_through_kernel(target)
puts(Holder.autoload?(:AlsoFromModuleMethod) == target)
puts(Object.autoload?(:AlsoFromModuleMethod).inspect)

puts(Holder::FromModuleMethod.loaded)
