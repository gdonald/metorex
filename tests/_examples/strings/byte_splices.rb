# `bytesplice` replaces a run of bytes in place, counting positions in bytes
# rather than in characters, and refuses an offset inside a character.
greeting = +"hello"
p greeting.bytesplice 0, 1, "j"
p greeting

p (+"hello").bytesplice 1..2, "HELLO", 0..1
p (+"hello").bytesplice 0, 0, "say "

japanese = +"こんにちは"
p japanese.bytesplice 0, 3, "xxx"

begin
  (+"こんにちは").bytesplice 1, 0, "x"
rescue IndexError => problem
  puts problem.message
end

begin
  (+"hello").bytesplice 6, 0, "x"
rescue IndexError => problem
  puts problem.message
end

begin
  (+"hello").bytesplice -6...-6, "x"
rescue RangeError => problem
  puts problem.message
end

begin
  "frozen".freeze.bytesplice 0, 1, "x"
rescue FrozenError => problem
  puts problem.message
end
