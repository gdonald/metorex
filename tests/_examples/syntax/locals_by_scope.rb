# Whether `name [1]` indexes a local or calls a method with an array depends
# on whether the scope it is written in bound the name before it. A `def`,
# `class` or `module` sees no local bound outside it, a block's locals end
# with the block, and a pattern, a named regexp group and a lambda parameter
# each bind a local.

def first
  foo = [10, 20]
  foo [1]
end

def foo(*args) = [:called, args]

def second
  foo [1]
end
p(first)
p(second)

bar = 5
class Holder
  def self.bar(*args) = [:class_bar, args]
  p(bar [2])
end

[1].each { |baz| baz }
def baz(*args) = [:method_baz, args]
p(baz [3])

if /(?<qux>\w+)/ =~ "hello"
  p(qux [1])
end

case [1, 2]
in [corge, _]
  p(corge -1)
end

module Spot
  grault = 1
end
def grault(*args) = [:grault, args]
p(grault [4])

reads_first = ->(held) { held [0] }
p(reads_first.call([7]))
