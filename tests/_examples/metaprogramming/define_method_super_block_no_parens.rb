module Walks
  def walk_pairs step
    [1, 2, 3].each { |value| yield value, value * step }
  end
end

class Batch
  include Walks

  [:walk_pairs].each do |name|
    define_method name do |*args, &block|
      return :no_block if block.nil?
      super(*args, &block)
    end
  end
end

p Batch.new.walk_pairs 10
Batch.new.walk_pairs(10) { |value, scaled| p [value, scaled] }

class Enumerator::Lazy
  [:each_cons].each do |name|
    define_method name do |*args, &block|
      super(*args, &block)
    end
  end
end

(1..4).lazy.each_cons(2) { |pair| p pair }
