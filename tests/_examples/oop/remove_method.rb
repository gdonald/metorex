class Parent
  def inherited_method
    :parent
  end
end

class Child < Parent
  def first
    :first
  end
  def second
    :second
  end
end

puts Child.send(:remove_method, :first, :second).inspect
puts Child.instance_methods(false).inspect
puts Child.new.inherited_method.inspect

begin
  Child.send(:remove_method, :inherited_method)
rescue NameError
  puts "inherited: NameError"
end

begin
  Child.send(:remove_method, :never_defined)
rescue NameError
  puts "missing: NameError"
end

puts Child.send(:remove_method).inspect

frozen = Module.new do
  def doomed
  end
end
frozen.freeze

begin
  frozen.send(:remove_method, :doomed)
rescue FrozenError
  puts "frozen: FrozenError"
end

puts (frozen.send(:remove_method) == frozen).inspect
puts Module.instance_method(:remove_method).arity.inspect
puts Module.public_instance_methods.include?(:remove_method).inspect
