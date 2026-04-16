cls = Class.new { def foo; 'foo'; end }
puts cls.new.foo

sub = Class.new(cls) { def baz; 'baz'; end }
obj = sub.new
puts obj.foo
puts obj.baz

m = Module.new { def bar; 'bar'; end }
puts m.class
