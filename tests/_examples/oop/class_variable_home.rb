module Shared
  @@count = :module

  def count
    @@count
  end

  def count=(value)
    @@count = value
  end
end

class Parent
  @@held = :parent

  def held
    @@held
  end

  def held=(value)
    @@held = value
  end
end

class Child < Parent; end

class Holder
  extend Shared
end

puts(Holder.count.to_s)
Holder.count = :written
puts(Holder.count.to_s)

Child.new.held = :from_child
puts(Parent.new.held.to_s)

parent = Class.new
subclass = Class.new(parent)
subclass.class_variable_set(:@@shared, :subclass)
parent.class_variable_set(:@@shared, :parent)
begin
  subclass.class_variable_get(:@@shared)
rescue RuntimeError => error
  puts(error.message.include?("is overtaken by").to_s)
end

begin
  eval("@@nowhere")
rescue RuntimeError => error
  puts(error.message)
end
