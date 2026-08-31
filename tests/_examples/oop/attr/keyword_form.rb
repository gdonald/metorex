class A
  attr :foo
end

a = A.new
a.instance_variable_set(:@foo, 42)
puts a.foo

class B
  attr :bar, true
end

b = B.new
b.bar = "hello"
puts b.bar

class C
  attr :a, :b
end

c = C.new
c.instance_variable_set(:@a, 1)
c.instance_variable_set(:@b, 2)
puts c.a
puts c.b

# `attr` answers the names it defined. A class body has its own local scope,
# so the value is printed there rather than assigned to an outer variable.

class D
  defined_names = attr :foo, 'bar'
  puts defined_names.inspect
end

class E
  defined_names = attr :baz, false
  puts defined_names.inspect
end

class F
  defined_names = attr :qux, true
  puts defined_names.inspect
end
