# RubyVM::InstructionSequence compiles a program, or names the method or
# block a Proc or a Method was compiled into. Metorex runs the syntax tree,
# so an instruction sequence holds the source and the node it parses to.
# MRI answers this way when it compiles with parse.y, as `ruby
# --parser=parse.y` does.
program = RubyVM::InstructionSequence.compile("total = 4 * 5\ntotal + 1")
p(program)
p([program.path, program.absolute_path, program.label, program.base_label, program.first_lineno])
p(program.eval)
p(program.trace_points)
puts(program.disasm.lines.first)

whole = RubyVM::InstructionSequence.compile_file(__FILE__)
p([whole.label, whole.path == __FILE__, whole.absolute_path == File.expand_path(__FILE__), whole.first_lineno])

named = RubyVM::InstructionSequence.new("40 + 2", "report.rb", "/reports/report.rb", 12)
p([named, named.path, named.absolute_path, named.first_lineno, named.eval])

def area(width, height) = width * height
measured = RubyVM::InstructionSequence.of(method(:area))
p([measured.label, measured.base_label, measured.first_lineno, measured.path == __FILE__])
begin
  measured.eval
rescue TypeError => error
  p(error.message)
end

doubled = proc { |amount| amount * 2 }
p(RubyVM::InstructionSequence.of(doubled).label)
def builder = proc { proc { :inner } }
p(RubyVM::InstructionSequence.of(builder.call).label)
module Reports
  FORMAT = proc { :format }
end
p(RubyVM::InstructionSequence.of(Reports::FORMAT).label)
p(RubyVM::InstructionSequence.of(method(:puts)))

children = []
RubyVM::InstructionSequence.compile("def a; end\nclass B; end\n[1].each { |x| x }").each_child do |child|
  children << [child.label, child.first_lineno]
end
p(children)

copy = RubyVM::InstructionSequence.load_from_binary(program.to_binary)
p(copy.eval)

[
  ["1 +", "<compiled>"],
  ["total = price *\n", "<compiled>"],
  ["break if ready", "loop.rb"]
].each do |source, file|
  RubyVM::InstructionSequence.compile(source, file)
rescue SyntaxError => error
  p(error.message)
end

begin
  RubyVM::InstructionSequence.allocate
rescue TypeError => error
  p(error.message)
end
p(RubyVM::InstructionSequence.compile_option[:debug_level])
