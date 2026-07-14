# `class ::Name` / `module ::Name` anchor the constant at the top level
# even when lexically nested. A bare superclass name resolves through the
# enclosing lexical scopes.
class Marker
end

module Wrapper
  class ::Marker
    TAG = :reopened_toplevel
  end

  module ::TopMod
    TAG = :toplevel_module
  end

  class Parent
    TAG = :parent
  end

  class Container
    class Child < Parent
    end
  end
end

puts Marker::TAG.inspect
puts TopMod::TAG.inspect
puts Wrapper.const_defined?(:Marker, false)
puts Wrapper.const_defined?(:TopMod, false)
puts Wrapper::Container::Child.superclass
