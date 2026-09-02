def innermost
  all = caller 0
  puts all[0].include? "#{__FILE__}:2:in 'Object#innermost'"
  puts all[1].include? "in 'Object#middle'"
  puts all[2].include? "in 'Object#outermost'"
  puts caller.length == all.length - 1
  puts caller(1..1).length == 1
  puts caller(2..-1) == all[2..-1]
  puts caller(100).inspect
  puts caller(all.length).inspect
end

def middle
  innermost
end

def outermost
  middle
end

outermost

reported = nil
runner = proc { reported = caller(0)[0] }
runner.call
puts reported.include? "in 'block in <main>'"

puts Kernel.private_instance_methods.include? :caller
