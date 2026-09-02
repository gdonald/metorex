plain = <<TEXT
first line
second line
TEXT
p(plain)

upcased = <<TEXT.upcase
shout
TEXT
p(upcased)

name = "world"
interpolated = <<TEXT
hello #{name}
TEXT
p(interpolated)

literal = <<'TEXT'
no #{interpolation}
TEXT
p(literal)

collected = []
collected << "shovel still works"
p(collected)
p(1 << 3)
