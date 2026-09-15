# `class << x` runs its body with the object's singleton class as self, so a
# bare `to_s` there is that class's own.
class Foo
  puts to_s
end

held = +"test"
named = class << held
  puts self.class.to_s
  to_s
end
puts named.start_with?("#<Class:").to_s

# A method cut from an object and installed elsewhere keeps the object it was
# bound to.
class Counter
  def count
    "counted"
  end
end

reader = Counter.new.method :count
Foo.send :define_method, :borrowed, reader.to_proc
puts Foo.new.borrowed

# A `def self.name` method belongs to the singleton class.
class Parent
  def self.greeting
    "hello"
  end
end
class Child < Parent; end
puts Child.singleton_class.instance_method(:greeting).owner.to_s
puts Parent.singleton_class.instance_methods(false).inspect
