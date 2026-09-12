# A block that names no parameters reads what it was handed as `_1` and `_2`,
# and a call written `m(a:)` passes what `a` names under that keyword.
doubled = [1, 2, 3].map { _1 * 2 }
p doubled

pairs = [[1, 10], [2, 20]].map { _1 }
p pairs

def totals(first:, second:)
  [first, second, first + second]
end

first = 4
second = 6
p totals(first:, second:)
p totals(first: 1, second:)

gathered = { first:, second: }
p gathered
