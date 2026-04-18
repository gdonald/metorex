a = Class.new
singleton_class = (class << a; self; end)
puts singleton_class.attached_object == a
