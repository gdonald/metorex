module Outer
  module Inner
    @recorded = {}
    def self.[] name
      @recorded[name]
    end
    def self.[]= name, value
      @recorded[name] = value
    end

    Inner[:body] = Module.nesting

    class << self
      Inner[:singleton] = Module.nesting
    end

    class Nested
      Inner[:nested_class] = Module.nesting
    end

    def self.from_method
      Module.nesting
    end
  end

  Inner[:outer_body] = Module.nesting
end

Outer::Inner[:top_level] = Module.nesting

puts Outer::Inner[:top_level].inspect
puts Outer::Inner[:outer_body].inspect
puts Outer::Inner[:body].inspect
puts Outer::Inner[:nested_class].inspect
puts (Outer::Inner[:singleton] == [Outer::Inner.singleton_class, Outer::Inner, Outer]).inspect
puts Outer::Inner.from_method.inspect
