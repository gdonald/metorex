# `singleton_method_added` fires for every way a singleton method arrives:
# `def obj.name`, a `class << obj` body, alias, and define_method.

object = Object.new

def object.singleton_method_added(name)
  puts("object gained #{name}")
end

def object.by_def
end

class << object
  def in_singleton_body
  end

  alias_method(:aliased, :in_singleton_body)

  define_method(:by_define_method) {}
end

object.define_singleton_method(:by_define_singleton_method) {}

class Host
  def self.singleton_method_added(name)
    puts "Host gained #{name}"
  end

  class << self
    def class_side
    end
  end

  def instance_side
  end
end

# A class body body value is the body's own last value, not a re-run of it.
counted = 0
adder = class << object
  counted += 1
  self
end
puts(counted.to_s)
puts(adder.instance_of?(Class).to_s)
