# A block answers the value of its last statement, and a last statement that
# answers nothing, such as an `if` whose branch did not run or a loop, makes
# it answer nil.
checks = {
  "assign" => proc { 1; x = 5 },
  "op assign" => proc { x = 1; x += 2 },
  "multiple assign" => proc { 1; a, b = 3, 4 },
  "if true" => proc { 1; 7 if true },
  "if false" => proc { 1; 7 if false },
  "unless" => proc { 1; 8 unless true },
  "if else" => proc { 1; if false then 2 else 3 end },
  "case" => proc { 1; case 2 when 3 then :x end },
  "case hit" => proc { 1; case 2 when 2 then :y end },
  "while" => proc { 1; while false do end },
  "until" => proc { 1; i = 0; until i > 2 do i += 1 end },
  "def" => proc { 1; def block_value_method = 1 },
  "begin" => proc { 1; begin; 4; end },
  "begin rescue" => proc { 1; begin; raise "x"; rescue; 6; end },
  "ivar" => proc { 1; @v = 9 },
  "constant" => proc { 1; BLOCK_VALUE_CONSTANT = 2 },
  "class" => proc { 1; class BlockValueClass; 3; end },
  "nested if" => proc { 1; if true then 2 if false end },
  "and" => proc { 1; nil && 3 },
  "puts" => proc { 1; puts },
}
checks.each { |label, block| p [label, block.call] }
lam = ->(x) { y = x; :z if false }
p lam.call 1
p [1, 2].map { |n| n * 2 if n > 1 }
