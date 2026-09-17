# A range written as a literal is frozen the moment it is made, so the only
# range whose ends can still be written is one `allocate` handed back.
held = Range.allocate
held.send :initialize, 3, 7, true

p held.begin
p held.end
p held.exclude_end?

# Both ends have to be comparable with each other.
begin
  Range.allocate.send :initialize, Object.new, Object.new
rescue ArgumentError => error
  puts error.message
end

# A range that is already built refuses to be written again.
begin
  (0..1).send :initialize, 1, 3
rescue FrozenError => error
  puts error.class
end
