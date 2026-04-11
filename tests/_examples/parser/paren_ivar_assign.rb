class Foo
  def bar
    (@x = 5).to_s
  end
end

puts Foo.new.bar
