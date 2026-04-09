add = lambda { |a, b| a + b }
puts add[3, 4]
puts add.call(10, 20)

double = lambda { |x| x * 2 }
puts double[21]

no_args = lambda { 99 }
puts no_args[]
