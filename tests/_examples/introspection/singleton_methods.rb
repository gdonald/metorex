plain = Object.new
puts plain.singleton_methods.inspect

widget = Object.new
def widget.polish
  :shiny
end
puts widget.singleton_methods.inspect

module Greeting
  def greet
  end
end
widget.extend(Greeting)
puts widget.singleton_methods.sort.inspect
puts widget.singleton_methods(false).inspect

class Parent
  def self.parent_class_method
  end
end

class Child < Parent
  def self.child_class_method
  end

  class << self
    def opened_on_child
    end

    private

    def hidden_class_method
    end
  end
end

puts Child.singleton_methods.sort.inspect
puts Child.singleton_methods(false).sort.inspect
puts Parent.singleton_methods.inspect

module Helper
  extend self

  def assist
    :assisted
  end
end
puts Helper.assist.inspect
puts Helper.singleton_methods.inspect

numbers = [1, 2, 3, 4, 5]
puts numbers[1..3].inspect
puts numbers[1...3].inspect
puts numbers[2..].inspect
puts numbers[..2].inspect
puts numbers[-2..].inspect
puts numbers[9..].inspect
