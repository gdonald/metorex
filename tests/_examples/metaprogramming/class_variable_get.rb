# Module#class_variable_get — written without parentheses. Reads a class
# variable by name, walking included modules and superclasses, and raises a
# NameError when the variable is not defined.
module Trackable
  @@tracked = :yes
end

class Counter
  @@count = 7
end

class Widget < Counter
  include Trackable
end

puts Counter.class_variable_get :@@count
puts Counter.class_variable_get "@@count"
puts Widget.class_variable_get :@@tracked
puts Widget.class_variable_get :@@count

c = Class.new
c.class_variable_set :@@local, "here"
puts c.class_variable_get :@@local

begin
  c.class_variable_get :@@missing
rescue NameError
  puts "missing raises NameError"
end
