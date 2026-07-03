class Basic; end

class Parent < Basic; end

module Mixin; end

class Child < Parent
  include Mixin
end

puts (Child <=> Parent).inspect
puts (Child <=> Basic).inspect
puts (Child <=> Mixin).inspect
puts (Child <=> Child).inspect
puts (Parent <=> Child).inspect
puts (Basic <=> Child).inspect
puts (Parent <=> Mixin).inspect
puts (Parent <=> Object.new).inspect
