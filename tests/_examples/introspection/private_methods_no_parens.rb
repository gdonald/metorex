module Helpers
  def mixed_in_helper
  end
  private :mixed_in_helper
end

class Parent
  def parent_secret
  end
  private :parent_secret

  class << self
    def parent_class_secret
    end
    private :parent_class_secret
  end
end

class Child < Parent
  include Helpers

  def child_secret
  end
  private :child_secret

  class << self
    def child_class_secret
    end
    private :child_class_secret
  end
end

def secrets names
  names.select { |name| name.to_s.end_with?("secret") }.sort
end

child = Child.new
puts secrets(child.private_methods(false)).inspect
puts secrets(child.private_methods).inspect
puts secrets(Child.private_methods(false)).inspect
puts secrets(Child.private_methods).inspect
puts secrets(child.private_methods(nil)).inspect

widget = Object.new
class << widget
  def singleton_secret
  end
  private :singleton_secret
end
puts secrets(widget.private_methods(false)).inspect

extended = Object.new
extended.extend Helpers
puts extended.private_methods.include? :mixed_in_helper

puts (/_secret\z/ =~ :child_secret).inspect
puts (/nope\z/ =~ :child_secret).inspect
puts (:child_secret !~ /_secret\z/).inspect
