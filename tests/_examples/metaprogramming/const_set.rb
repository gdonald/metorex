# Module#const_set — written without parentheses where possible. Covers
# return value, anonymous-module naming (including the cascade into
# nested anonymous modules), the redefinition warning, name validation,
# and the frozen check.
mod = Module.new
puts mod.name.inspect

a, b, c = Module.new, Module.new, Module.new
a::B = b
a::B::C = c

Object.const_set :NamedRoot, a
puts a.name
puts b.name
puts c.name

inner = Module.new
scoped = Module.new
scoped.const_set :Inner, inner
puts inner.name.end_with?("::Inner")

puts NamedRoot.const_set(:VALUE, 41).inspect

begin
  NamedRoot.const_set "lowercase", 1
rescue NameError
  puts "NameError"
end

frozen = Module.new.freeze
begin
  frozen.const_set :Foo, 1
rescue FrozenError
  puts "FrozenError"
end

Object.send :remove_const, :NamedRoot
