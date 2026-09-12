# A multiple assignment written inside a group answers its right-hand side.
p((a, b = 1, 2))
p([a, b])

p((first, second, third = 1))
p([first, second, third])

# `defined?` reports on an assignment without carrying it out.
p(defined?(counted = 1))
p(defined?(@held = 2))
p(defined?($named = 3))
p(defined?(counted %= 2))
held = []
p(defined?(held[0] = 1))
p(held)
