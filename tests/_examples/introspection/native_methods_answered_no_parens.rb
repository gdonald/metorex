# The methods core classes answer natively are reported the way any other
# method is, by respond_to?, defined? and method_defined?.
list = [1, 2]
p list.respond_to?(:each), list.respond_to?(:size), list.respond_to?(:+)
p defined?(list.each), defined?(list.no_such_method)
p({ a: 1 }.respond_to?(:each_pair), "text".respond_to?(:chomp), :name.respond_to?(:to_proc))
p 7.respond_to?(:between?), 1.5.respond_to?(:nan?), nil.respond_to?(:to_a), true.respond_to?(:&)
p Object.new.respond_to?(:instance_exec), Object.new.respond_to?(:chomp), Object.new.respond_to?(:puts)
p Array.method_defined?(:each), Array.method_defined?(:each, false), Class.new(Array).method_defined?(:each, false)
p Integer.public_method_defined?(:between?), Comparable.method_defined?(:clamp), Array.private_method_defined?(:each)
p Class.new.method_defined?(:__send__)
