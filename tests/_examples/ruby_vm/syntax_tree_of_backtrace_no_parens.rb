# Each location in a backtrace finds the call node written at it, the call
# a native method such as Array#each stands at among them.
def calls_missing = nil.no_such_method

[1].each do
  [2].each { calls_missing }
rescue NoMethodError => raised
  raised.backtrace_locations.each do |location|
    node = RubyVM::AbstractSyntaxTree.of location
    p [location.label, node.type, node.first_lineno, node.first_column]
  end
end

begin
  [3].map { |number| number / 0 }
rescue ZeroDivisionError => raised
  node = RubyVM::AbstractSyntaxTree.of raised.backtrace_locations.first
  p [node.type, node.first_column, node.last_column]
end
