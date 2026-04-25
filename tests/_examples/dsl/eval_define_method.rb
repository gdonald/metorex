code = 'define_method(:boom) { :bam }'
result = eval code, TOPLEVEL_BINDING
puts "eval result: #{result.inspect}"

methods = Object.methods(true)
puts "Object has boom? #{methods.include?(:boom)}"

# Try calling it
begin
  puts "calling: #{boom.inspect}"
rescue NoMethodError => e
  puts "no boom: #{e.message}"
end
