# A value's hash is the same every time this process asks, and a different
# number in another process, so a table cannot be filled with values chosen
# to collide.
require "rbconfig"

values = ["14", "10**30", "3.14", "Rational(1, 2)", "Complex(1, 2)", "'abc'", ":a", "[1, 2]", "{a: 1}"]
values.each do |written|
  here = eval(written).hash
  again = eval(written).hash
  there = IO.popen([RbConfig.ruby, "-e", "print((#{written}).hash)"], &:read)
  p([written, here.class, here == again, here.to_s != there])
end
