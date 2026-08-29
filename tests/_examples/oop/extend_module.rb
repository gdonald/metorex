module Namespace
end

module Namespace::Taggable
  def tag
    "tagged"
  end
end

class Direct
  extend Namespace::Taggable
end

puts Direct.tag
puts Direct.kind_of?(Namespace::Taggable)

anonymous = Class.new do
  extend Namespace::Taggable
end

puts anonymous.tag
puts anonymous.kind_of?(Namespace::Taggable)

single = Object.new
single.extend Namespace::Taggable
puts single.tag
puts single.kind_of?(Namespace::Taggable)
puts Object.new.kind_of?(Namespace::Taggable)

begin
  Object.new.extend(Class.new)
rescue TypeError => error
  puts error.class
end

puts Object.new.method(:extend).arity
