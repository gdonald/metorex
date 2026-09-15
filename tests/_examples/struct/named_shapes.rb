# `Struct.new` names a constant when its first argument is a String, refuses a
# repeated member, and reads keywords as members when nothing else is given.
Struct.new "Waypoint", :name, :miles
p Struct::Waypoint.name
p Struct::Waypoint.new("summit", 12).name

Point = Struct.new :across, :down
p Point.new(1, 2).to_a
p Point.new(across: 1).to_a

Keyed = Struct.new :name, :legs, keyword_init: true
p Keyed.new(name: "elefant", legs: 4).to_a
p Keyed.new({ name: "mouse", legs: 4 }).to_a

begin
  Struct.new :foo, :foo
rescue ArgumentError => problem
  puts problem.message
end

begin
  Struct.new "lowercase", :foo
rescue NameError => problem
  puts problem.class
end

begin
  Keyed.new name: "elefant", missing: 1
rescue ArgumentError => problem
  puts problem.message
end
