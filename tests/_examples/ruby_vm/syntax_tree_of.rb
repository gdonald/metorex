# RubyVM::AbstractSyntaxTree.of reads again the file a Proc, a Method or a
# backtrace location was written in and answers the node written there.
# MRI answers this way when it compiles with parse.y, as `ruby
# --parser=parse.y` does, and refuses when it compiles with Prism.
add = proc { |left, right| left + right }
twice = ->(amount) { amount * 2 }

def total(amounts) = amounts.sum

p(RubyVM::AbstractSyntaxTree.of(add))
p(RubyVM::AbstractSyntaxTree.of(twice))
p(RubyVM::AbstractSyntaxTree.of(method(:total)))
p(RubyVM::AbstractSyntaxTree.of(method(:total)).children[1].children[0])
p(RubyVM::AbstractSyntaxTree.of(add, keep_script_lines: true).source)
p(RubyVM::AbstractSyntaxTree.of(twice, keep_tokens: true).tokens.map { |token| token[2] })

begin
  puts([nil].first.upcase)
rescue NoMethodError => error
  location = error.backtrace_locations.first
  node = RubyVM::AbstractSyntaxTree.of(location)
  p(node)
  p(RubyVM::AbstractSyntaxTree.node_id_for_backtrace_location(location) == node.node_id)
end
p(RubyVM::AbstractSyntaxTree.of(caller_locations(0).first))

p(RubyVM::AbstractSyntaxTree.of(method(:puts)))
begin
  RubyVM::AbstractSyntaxTree.of(1)
rescue TypeError => error
  p(error.message)
end
begin
  RubyVM::AbstractSyntaxTree.of(eval("proc { 1 }"))
rescue ArgumentError => error
  p(error.message)
end
