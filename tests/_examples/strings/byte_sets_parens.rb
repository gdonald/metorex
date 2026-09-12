# A run of bytes spliced into an ASCII literal keeps its encoding, and a
# `tr`-style set whose bytes are not characters in that encoding is refused.
high = [0xFF].pack("C")
spliced = "\x00 - #{high}"
p(spliced.bytes)
p(spliced.encoding)
p(spliced.valid_encoding?)

as_text = spliced.force_encoding("utf-8")
p(as_text.valid_encoding?)

begin
  "hello".delete(as_text)
rescue ArgumentError => problem
  puts(problem.message)
end

begin
  "hello".delete("h-e")
rescue ArgumentError => problem
  puts(problem.message)
end

p("hello".delete("l"))
