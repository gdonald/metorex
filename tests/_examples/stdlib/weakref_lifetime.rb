# A WeakRef reaches its object while something else holds it, and raises
# once nothing does. An Integer is never freed, so a reference to one stays.
require "weakref"

held = Object.new
ref = WeakRef.new(held)
p(ref.weakref_alive?)
p(ref.__getobj__.equal?(held))

def make_reference(depth = 10)
  depth > 0 ? make_reference(depth - 1) : WeakRef.new(Object.new)
end

references = Array.new(100) { make_reference }
dead = nil
100.times do
  GC.start
  dead = references.find { |reference| !reference.weakref_alive? }
  break if dead
end
p(dead.nil?)
begin
  dead.__getobj__
rescue WeakRef::RefError => error
  p(error.message)
end
p(WeakRef.new(5).weakref_alive?)
