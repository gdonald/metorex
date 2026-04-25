puts :boom
puts(:boom)

def have_method(name, include_super = true)
  "matcher for #{name.inspect}"
end

x = have_method(:boom)
puts x

y = have_method :boom
puts y
