# Module#class_variable_set / class_variable_defined? — written without
# parentheses. A class variable assigned inside `class << self` attaches to
# the lexical enclosing class, and lookup walks included modules and
# superclasses.
class Counter
  @@count = 0

  class << self
    @@meta = :info
  end
end

module Trackable
  @@tracked = true
end

class Widget < Counter
  include Trackable
end

puts Counter.class_variable_defined? :@@count
puts Counter.class_variable_defined? "@@count"
puts Counter.class_variable_defined? :@@missing
puts Counter.class_variable_defined? :@@meta

Counter.class_variable_set :@@count, 5
puts Counter.class_variable_defined? :@@count

puts Widget.class_variable_defined? :@@tracked
puts Widget.class_variable_defined? :@@count

c = Class.new
c.class_variable_set :@@local, "here"
puts c.class_variable_defined? :@@local
puts c.class_variable_defined? :@@other
