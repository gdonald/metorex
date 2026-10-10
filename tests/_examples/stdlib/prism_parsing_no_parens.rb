require "prism"
require "stringio"

source = "def total(items) = items.sum { |item| item.price * 2 }\nputs(total([]))\n"
result = Prism.parse(source)
root = result.value
p Prism::VERSION
p result.success?
p root.statements.body.map(&:type)

definition = root.statements.body.first
p definition.name
p definition.parameters.requireds.map(&:name)
p [definition.location.start_line, definition.location.end_column]
p definition.name_loc.slice

class CallNames < Prism::Visitor
  attr_reader :names

  def initialize
    @names = []
  end

  def visit_call_node(node)
    @names << node.name
    super
  end
end

names = CallNames.new
root.accept(names)
p names.names
p Prism::Pattern.new("CallNode[name: :sum]").scan(root).map(&:name).to_a

p Prism.lex("rate = 7").value.map { |token, _state| [token.type, token.value] }
p Prism.parse("def close").errors.map { |error| [error.type, error.location.start_offset] }
p Prism.parse_success?("x = 1")
p Prism.parse_failure?("x =")
p Prism.parse_comments("# shipping\nrate = 7 # per pound\n").map(&:slice)
p Prism.parse_lex("rate = 7").value.last.size
p Prism.parse("rate\nfee", line: 10).value.statements.body.map { |node| node.location.start_line }
p Prism.parse("rate = 7", scopes: [[:fee]]).value.locals
p Prism.parse_file(__FILE__).value.statements.body.size
p Prism.parse_stream(StringIO.new("a = 1\nb = 2\n__END__\nunparsed\n")).value.statements.body.size
p Prism.dump("1").bytesize
p Prism::Translation::Ripper.sexp("rate + 1")
p Prism::StringQuery.new("rate").local?
p Prism::StringQuery.new("Rate").constant?
