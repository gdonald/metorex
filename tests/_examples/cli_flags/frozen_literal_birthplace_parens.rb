# Under --debug-frozen-string-literal a string literal remembers where it was
# written, and the refusal to change it names that place.
held = 'written here'
begin
  held.<<(' and more')
rescue FrozenError => refused
  puts(refused.message)
end
