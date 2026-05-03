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

result_one = nil
class D
  result_one = (attr :foo, 'bar')
end
puts result_one.inspect

result_two = nil
class E
  result_two = (attr :baz, false)
end
puts result_two.inspect

result_three = nil
class F
  result_three = (attr :qux, true)
end
puts result_three.inspect
