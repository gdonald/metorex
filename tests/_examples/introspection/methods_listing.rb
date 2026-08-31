module Greeting
  def greet
    "hello"
  end
end

class Widget
  def spin
    :spinning
  end
end

widget = Widget.new
puts widget.methods(false).inspect

def widget.polish
  :shiny
end
puts widget.methods(false).inspect

widget.extend(Greeting)
puts widget.methods(false).inspect
puts widget.methods.include?(:greet)
puts widget.methods.include?(:spin)

class << widget
  def buff
    :buffed
  end

  private

  def secret
    :hidden
  end
end
puts widget.methods(false).inspect

singleton = class << widget
  self
end
singleton.send(:undef_method, :polish)
puts widget.methods(false).inspect

class Parent
  def inherited_method
    :from_parent
  end
end

class Child < Parent
  undef_method :inherited_method
end
puts Child.new.methods.include?(:inherited_method)
puts Parent.new.methods.include?(:inherited_method)

puts :symbol.class.name
puts (String === :symbol).inspect
puts :symbol.length

left = [1, 2, 3, 2]
right = [2, 3, 4]
puts (left & right).inspect
puts (left | right).inspect
