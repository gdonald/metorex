# A literal in a source that says nothing about frozen string literals is
# handed back with notice that a later release will freeze it.
held = "chilled"
p(held.frozen?)

# `+@` on such a literal answers a copy that carries no such notice, where
# `-@` answers the one frozen string every place writing it shares.
mutable = +held
p(mutable.frozen?)
p(mutable.equal?(held))
p((-held).frozen?)

# `freeze` on a literal answers that same shared string.
p("shared".freeze.equal?("shared".freeze))

# Two string literals written next to each other are one string.
p("still" "+chilled")
p("a" "b" "c")
