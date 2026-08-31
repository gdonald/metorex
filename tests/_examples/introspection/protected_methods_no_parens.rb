module Helpers
  def mixed_in_guard
  end
  protected :mixed_in_guard
end

class Parent
  def parent_guard
  end
  protected :parent_guard

  class << self
    def parent_class_guard
    end
    protected :parent_class_guard
  end
end

class Child < Parent
  include Helpers

  def child_guard
  end
  protected :child_guard

  class << self
    def child_class_guard
    end
    protected :child_class_guard
  end
end

def guards names
  names.select { |name| name.to_s.end_with?("guard") }.sort
end

child = Child.new
puts guards(child.protected_methods(false)).inspect
puts guards(child.protected_methods).inspect
puts guards(Child.protected_methods(false)).inspect
puts guards(child.protected_methods(nil)).inspect

widget = Object.new
class << widget
  def singleton_guard
  end
  protected :singleton_guard
end
puts guards(widget.protected_methods(false)).inspect

extended = Object.new
extended.extend Helpers
puts extended.protected_methods.include? :mixed_in_guard
puts child.protected_methods(false).include? :mixed_in_guard
