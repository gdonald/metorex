# Reading this file has the object space record where each object was made,
# and has `p` name that place beside the object it writes.
require "objspace"

ObjectSpace.__trace_allocations__ true
STDERR.puts "objspace/trace is enabled"

module Kernel
  # The same as `p`, with the place an object was made written after it. An
  # object made before the tracing began has no place to name, so it is
  # written on its own.
  def p(*objects)
    objects.each do |object|
      place = ObjectSpace.__allocation_site__ object
      STDOUT.puts place.nil? ? object.inspect : "#{object.inspect} @ #{place}"
    end
    case objects.length
    when 0 then nil
    when 1 then objects.first
    else objects
    end
  end
end
