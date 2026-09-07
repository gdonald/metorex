require_relative "source_file_lib/reporter"

puts Reporter.own_file
puts Reporter.file_from_a_block

module Evaluated
end

puts Evaluated.class_eval "__FILE__", "custom.rb", 7
puts Evaluated.class_eval "__LINE__", "custom.rb", 7
