def ARGF.gets
  "stubbed line"
end

puts gets
puts ARGF.gets
puts Kernel.private_instance_methods.include?(:gets)
puts ARGF.class
