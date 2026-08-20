class Base
  def greet
    "hello"
  end

  def farewell
    "bye"
  end
end

class Quiet < Base
  undef greet, :farewell
end

puts Base.new.greet
puts Quiet.new.respond_to?(:greet)
puts Quiet.new.respond_to? :farewell

begin
  Quiet.new.greet
rescue NoMethodError
  puts "greet undefined"
end
