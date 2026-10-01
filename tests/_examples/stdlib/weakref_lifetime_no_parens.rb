# A WeakRef reaches its object while something else holds it, and raises
# once nothing does. An Integer is never freed, so a reference to one stays.
require "weakref"

held = Object.new
ref = WeakRef.new held
p ref.weakref_alive?
p ref.__getobj__.equal? held

def make_reference depth = 10
  return WeakRef.new Object.new if depth.zero?
  make_reference depth - 1
end

references = Array.new 100 do
  make_reference
end
dead = nil
100.times do
  GC.start
  dead = references.find { |reference| !reference.weakref_alive? }
  break if dead
end
p dead.nil?
begin
  dead.__getobj__
rescue WeakRef::RefError => error
  p error.message
end
number = WeakRef.new 5
p number.weakref_alive?
