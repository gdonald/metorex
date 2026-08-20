$order = []

module Tracker
  def self.included(base)
    $order << [:included, base.name]
  end
end

module SelfExtender
  def self.included(base)
    base.extend(self)
  end

  def helper
    :helped
  end
end

module Chained
  class << Chained
    def included(base)
      $order << :chained
      super
    end
  end
end

class Host
  include(Tracker)
end

class Helped
  include(SelfExtender)
end

class Linked
  include(Chained)
end

puts($order.inspect)
puts(Helped.helper.inspect)
puts(Module.new.private_methods.include?(:included).inspect)
