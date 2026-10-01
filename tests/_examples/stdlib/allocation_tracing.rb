# Where ObjectSpace says an object was made while allocation tracing is on:
# the file, the line, and the class and name of the method running there.
require 'objspace'

def site(object)
  [ObjectSpace.allocation_sourcefile(object) == __FILE__,
   ObjectSpace.allocation_sourceline(object),
   ObjectSpace.allocation_class_path(object),
   ObjectSpace.allocation_method_id(object)]
end

class Shelf
  def stock
    [Object.new, [1], "label", { size: 2 }]
  end

  def self.build
    [Object.new]
  end
end

module Labeled
  def label
    "tag"
  end
end

class Box
  include Labeled
end

made = ObjectSpace.trace_object_allocations do
  Shelf.new.stock.each { |object| p(site(object)) }
  p(site(Shelf.build.first))
  p(site(Box.new.label))
  top = [1, 2]
  p(site(top))
  p(ObjectSpace.allocation_generation(top).is_a?(Integer))
  [nil, 42, :name].each { |object| p(site(object)) }
  ObjectSpace.trace_object_allocations_clear
  p(site(top))
  :finished
end
p(made)

kept = nil
ObjectSpace.trace_object_allocations_start
ObjectSpace.trace_object_allocations_start
ObjectSpace.trace_object_allocations_stop
kept = Object.new
ObjectSpace.trace_object_allocations_stop
ObjectSpace.trace_object_allocations_stop
p(ObjectSpace.allocation_sourceline(kept))
p(ObjectSpace.allocation_sourceline(Object.new))
