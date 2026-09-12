# A backtrace names a method on a class with a dot and one on an instance
# with a hash, and a block run against another receiver sits where the call
# was made.
module Held
  class Raiser
    def self.from_a_class_method
      raise("from a class method")
    end

    def from_an_instance_method
      raise("from an instance method")
    end
  end
end

def labels_of(raised)
  raised.backtrace.first(2).map { |entry| entry.split(":in ").last }
end

begin
  Held::Raiser.from_a_class_method
rescue RuntimeError => raised
  p(labels_of(raised))
end

begin
  Held::Raiser.new.from_an_instance_method
rescue RuntimeError => raised
  p(labels_of(raised))
end

class Runner
  def self.protect(&block)
    Object.new.instance_exec(&block)
  end
end

seen = nil
Runner.protect { seen = caller(1).first.split(":in ").last }
p(seen)
