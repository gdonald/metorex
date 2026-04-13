# case/when as last statement in method body should return value via ControlFlow::Value
def grade(score)
  case score
  when 90..100
    "A"
  when 80..89
    "B"
  when 70..79
    "C"
  else
    "F"
  end
end
puts grade(95)
puts grade(82)
puts grade(75)
puts grade(50)
