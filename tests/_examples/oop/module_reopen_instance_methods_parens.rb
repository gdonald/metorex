class Module
  def described
    "module method on #{name}"
  end
end

class Class
  def built
    "class method on #{name}"
  end
end

module Mixin
end

class Widget
end

puts(Mixin.described)
puts(Widget.described)
puts(Widget.built)
puts(Widget.respond_to?(:described))
