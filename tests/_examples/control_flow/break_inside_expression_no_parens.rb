# A `break` written inside an expression in the block of a built-in
# iterator ends that iterator with the value it carries, as one written as
# a statement does.
picked = [1, 2, 3].each { |number| number == 2 ? (break :middle) : number }
p picked

counted = 5.times { |index| index > 1 && (break index * 10) }
p counted

doubled = [4, 5, 6].map { |number| [number == 5 ? (break :stopped) : number] }
p doubled

nested = [1, 2].map do |outer|
  [10, 20].each { |inner| inner == 20 ? (break) : inner }
  outer
end
p nested
