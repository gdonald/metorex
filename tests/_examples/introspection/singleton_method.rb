widget = Object.new
def widget.polish
  :shiny
end

polish = widget.singleton_method(:polish)
puts polish.class.name
puts polish.call.inspect

included = Module.new do
  def from_include
    :included
  end
end
widget.singleton_class.include(included)
puts widget.singleton_method(:from_include).call.inspect

prepended = Module.new do
  def from_prepend
    :prepended
  end
end
widget.singleton_class.prepend(prepended)
puts widget.singleton_method(:from_prepend).call.inspect

extension = Module.new do
  def from_extend
    :extended
  end
end
other = Object.new
other.extend(extension)
puts other.singleton_method(:from_extend).call.inspect

class Widget
  def instance_level
    :from_class
  end
end

plain = Widget.new
puts plain.instance_level.inspect
begin
  plain.singleton_method(:instance_level)
rescue NameError => error
  puts error.class.name
end

begin
  Object.new.singleton_method(:never_defined)
rescue NameError => error
  puts error.class.name
end

class Registry
  def self.lookup
    :found
  end
end
puts Registry.singleton_method(:lookup).call.inspect
