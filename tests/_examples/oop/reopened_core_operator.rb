module Counting; end

class Integer
  prepend Counting
end

$additions = []

sum = -> { 1 + 2 }
p sum.call

Counting.module_eval do
  def +(other)
    $additions.push other
    super other
  end
end

p sum.call
p $additions

class String
  def -(other)
    sub other, ""
  end
end

p "hello world" - " world"
