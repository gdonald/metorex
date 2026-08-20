$order = []

module Tracker
  def self.extend_object obj
    $order << :extend_object
  end

  def self.extended obj
    $order << :extended
  end
end

module Plain
  def self.extended obj
    $order << [:plain_extended, obj.class]
  end
end

Object.new.extend Tracker
Object.new.extend Plain
puts $order.inspect

names = Module.new.private_methods
puts names.include?(:extended).inspect
puts names.include?(:extend_object).inspect
puts Class.new.private_methods.include?(:extended).inspect
