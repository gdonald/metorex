puts Kernel.private_instance_methods(false).include?(:gets)
puts Kernel.private_instance_methods(false).include?(:readline)
puts Kernel.private_instance_methods(false).include?(:readlines)

puts EOFError.superclass.name
puts EOFError.ancestors.include?(StandardError)

# With no input to read, `gets` answers nil and `readlines` is empty, while
# `readline` refuses and raises.
puts gets.inspect
puts readlines.inspect

begin
  readline
rescue EOFError => error
  puts "#{error.class}: #{error.message}"
end

begin
  readline 1
rescue RuntimeError => error
  puts error.message
end
