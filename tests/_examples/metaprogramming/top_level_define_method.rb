eval("define_method(:boom) { :bam }", TOPLEVEL_BINDING)
puts Object.new.methods.include?(:boom)
result = eval("define_method(:boing) { :bong }", TOPLEVEL_BINDING)
puts result
Object.send(:remove_method, :boom)
Object.send(:remove_method, :boing)
puts Object.new.methods.include?(:boom)
puts Object.new.methods.include?(:boing)
