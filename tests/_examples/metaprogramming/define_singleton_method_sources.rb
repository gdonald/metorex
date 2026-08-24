class Catalog
  LIMIT = 3

  define_singleton_method(:listing, self.method(:constants))
end

puts Catalog.listing.inspect
puts Catalog.methods.include?(:listing)

class Parent
  def self.greeting
    "from parent"
  end
end

class Child < Parent
end

unbound = Parent.method(:greeting).unbind
Child.send :define_singleton_method, :inherited_greeting, unbound
puts Child.inherited_greeting

begin
  Parent.inherited_greeting
rescue NoMethodError
  puts "not defined on the parent"
end

target = Object.new
target.define_singleton_method(:speak) { "hello" }
puts target.speak
puts target.methods.include?(:speak)

frozen = Object.new
frozen.freeze

begin
  frozen.define_singleton_method(:speak) { "hello" }
rescue FrozenError => error
  puts error.class
end
