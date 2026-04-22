# `Class#subclasses` — returns the direct subclasses of a class.
# Each subclass, whether declared via `class X < Base` or `Class.new(Base)`,
# is tracked as a weak reference so GC'd classes drop off the list.

class Base
end

class Alpha < Base
end

class Beta < Base
end

anon = Class.new(Base)

puts Base.subclasses.size
# Named subclasses only (the anonymous Class.new returns nil for `.name`).
named = Base.subclasses.sort_by { |c| c.name.to_s }.select { |c| !c.name.nil? }
named.each { |c| puts c.name }
puts anon.name.inspect
