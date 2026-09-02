class Module
  def const_added(name)
    puts("added #{name} to #{self.name || self.inspect}")
  end
end

module TopLevelModule
end

class TopLevelClass
end

TopLevelConstant = 1

module Outer
  Inner = 2

  module Nested
  end
end

AnonymousBound = Module.new
puts(AnonymousBound.name)
