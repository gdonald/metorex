# Module#class_exec / module_exec — block evaluated in the receiver's context,
# with the caller's arguments passed to the block. Written without parentheses
# wherever possible.

class Gadget
end

# Defines an instance method on the receiver.
Gadget.class_exec do
  def name
    "gadget"
  end
end
puts Gadget.new.name

# Arguments are passed straight through to the block parameters.
puts Gadget.class_exec(6, 7) { |a, b| a * b }

# Returns the block's last value.
puts Gadget.class_exec { 1 + 1 }

# module_exec is the alias and behaves the same on a Module.
module Marker
end
Marker.module_exec("tag") do |label|
  define_method(:label) { label }
end
class Holder
  include Marker
end
puts Holder.new.label

# An instance of a Module subclass is itself a module and answers class_exec.
class NamedModule < Module
end
puts NamedModule.new.class_exec { 40 + 2 }
