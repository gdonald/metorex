# A module included into Object, as mkmf's MakeMakefile is, gives every object
# its methods, while an exception, nil, true and false keep answering their own.
module Reporting
  def message(*parts) = "reported #{parts.join}"
  def to_s = "reported text"
  def inspect = "reported inspect"
end

include(Reporting)

class Unreadable < StandardError
  def message = "own message"
end

class Shelf
end

error = RuntimeError.new("disk full")
p(error.message)
p(error.to_s)
p(error.full_message(highlight: false).include?("disk full"))
p(ArgumentError.new("bad size").message)
p(Unreadable.new("hidden").message)
p(Object.new.message("a", "b"))
p(5.message)
p(error.respond_to?(:message))
p([nil.to_s, true.to_s, false.to_s])
puts(nil.inspect, true.inspect, false.inspect)
puts(Shelf.new.to_s)
puts("#{nil}|#{true}|#{false}")
