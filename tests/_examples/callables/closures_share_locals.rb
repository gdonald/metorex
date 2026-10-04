# A block reads and writes the locals of the scope it was written in, as do
# blocks inside it and code it evaluates. A method or class body sees none of
# the program's top-level locals, while constants reach everywhere.

LIMIT = 3
total = 0
3.times { |step| total += step }
p(total)

counter = 0
bump = -> { counter += 1 }
2.times { bump.call }
p(counter)

[1, 2].each do |outer|
  [10].each { |inner| total += outer * inner }
end
p(total)

seen = :outer
p([1].map { eval("seen") })
p(proc { binding.local_variables.include?(:seen) }.call)

shadowed = 1
[5].each { |shadowed| shadowed += 1 }
p(shadowed)

hidden = :top_level
def reach
  hidden
rescue NameError => trouble
  [trouble.class, LIMIT]
end
p(reach)

class Holder
  begin
    hidden
  rescue NameError => trouble
    p(trouble.class)
  end
  p(LIMIT)
end
