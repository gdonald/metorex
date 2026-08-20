module Shared
  def from_module
  end
end

class Parent
  include Shared

  def public_parent
  end

  def retired
  end

  protected

  def protected_parent
  end

  private

  def private_parent
  end
end

class Parent
  undef_method :retired
end

class Child < Parent
  def public_child
  end
end

puts Parent.instance_methods(false).inspect
puts Parent.public_instance_methods(false).inspect
puts Parent.protected_instance_methods(false).inspect
puts Parent.private_instance_methods(false).inspect
puts Child.instance_methods(false).inspect
puts Child.instance_methods.include?(:from_module).inspect
puts Child.instance_methods.include?(:retired).inspect

class Caller
  def try_protected obj
    obj.protected_parent
  end
end

begin
  Parent.new.protected_parent
rescue NameError => error
  puts error.class
end
