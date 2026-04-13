# Test case/in pattern matching in various contexts

# 1. case/in in a method body (covers execute_method_body ControlFlow::Value)
def match_num(x)
  case x
  in 1
    "one"
  in 2
    "two"
  end
end
puts match_num(1)
puts match_num(2)

# 2. case/in inside a block (covers execute_block_body ControlFlow::Value)
result = [1, 2].map do |n|
  case n
  in 1
    "one"
  in 2
    "two"
  end
end
puts result[0]
puts result[1]
