# What the object space can say about a program: the readings the collector
# reports, how much has been built, and what one object refers to.
readings = GC.stat
puts readings.is_a?(Hash).to_s
puts readings[:count].is_a?(Integer).to_s
puts GC.stat(:count).is_a?(Integer).to_s

require "objspace"
made = Class.new
before = ObjectSpace.memsize_of_all made
made.new
puts (ObjectSpace.memsize_of_all(made) > before).to_s

held = Object.new
box = ["a", held]
reachable = ObjectSpace.reachable_objects_from box
puts reachable.include?(held).to_s
puts reachable.include?(Array).to_s
p ObjectSpace.reachable_objects_from 42
