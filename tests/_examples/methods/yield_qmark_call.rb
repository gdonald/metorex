class Foo
  def yield?(invert = false)
    !invert
  end

  def check
    yield?(false)
  end
end

puts Foo.new.check
