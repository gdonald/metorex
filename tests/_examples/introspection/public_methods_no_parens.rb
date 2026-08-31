module Helpers
  def mixed_in_open
  end
end

class Parent
  def parent_open
  end

  def parent_shut
  end
  private :parent_shut

  def self.parent_class_open
  end
end

class Child < Parent
  include Helpers

  def child_open
  end

  def child_guarded
  end
  protected :child_guarded

  def self.child_class_open
  end
end

def opens names
  names.select { |name| name.to_s.include?("open") }.sort
end

child = Child.new
puts opens(child.public_methods(false)).inspect
puts opens(child.public_methods).inspect
puts opens(Child.public_methods(false)).inspect
puts opens(child.public_methods(nil)).inspect
puts child.public_methods.include? :child_guarded
puts child.public_methods.include? :parent_shut

quotient = 13.divmod 4
puts quotient.inspect
negative_divisor = 13.divmod(-4)
puts negative_divisor.inspect
minus = -13
negative_value = minus.divmod 4
puts negative_value.inspect
puts 1.public_methods.include? :divmod

begin
  1.divmod 0
rescue ZeroDivisionError => error
  puts "#{error.class}: #{error.message}"
end
