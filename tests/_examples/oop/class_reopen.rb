class Foo
  def bar
    "bar"
  end
end

class Foo
  def baz
    "baz"
  end
end

f = Foo.new
puts f.bar
puts f.baz
