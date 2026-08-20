def guard(condition)
  yield if condition.call
end

guard -> { true } do
  puts "ran without parentheses"
end

guard(-> { true }) do
  puts "ran with parentheses"
end

guard -> { false } do
  puts "skipped"
end

def apply(fn, value)
  fn.call value
end

puts apply -> n { n * 2 }, 21
puts apply(-> n { n * 2 }, 21)
