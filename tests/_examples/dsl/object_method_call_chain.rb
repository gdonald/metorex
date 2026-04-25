def have_method(name, include_super = true)
  "matcher for #{name.inspect}"
end

class Object
  def my_should(matcher)
    puts "matcher class: #{matcher.class}"
    puts "matcher value: #{matcher.inspect}"
  end
end

Object.my_should have_method :boom
