# RubyVM::AbstractSyntaxTree keeps a program's lines and tokens when asked
# to, so a node can answer the text it was written as and the tokens it
# covers. A program it refuses names the line at fault with a caret.
source = "total = price * 2\nputs(total) # report\n"

plain = RubyVM::AbstractSyntaxTree.parse(source)
p plain.script_lines
p plain.source
p plain.tokens
p plain.all_tokens

kept = RubyVM::AbstractSyntaxTree.parse(source, keep_script_lines: true)
p kept.script_lines
p kept.source
assignment = kept.children[2].children[0]
p assignment.source
p assignment.children[1].source
p assignment.children[1].children[2].source

tokened = RubyVM::AbstractSyntaxTree.parse(source, keep_tokens: true)
tokened.all_tokens.each { |token| p(token) }
p tokened.children[2].children[1].tokens

heredoc = RubyVM::AbstractSyntaxTree.parse("text = <<~END # note\n  line\nEND\n", keep_tokens: true)
heredoc.all_tokens.each { |token| p(token) }

def each_node(node, &visit)
  return unless node.is_a?(RubyVM::AbstractSyntaxTree::Node)

  visit.call(node)
  node.children.each { |child| each_node(child, &visit) }
end
ids = []
each_node(kept) { |node| ids << node.node_id }
p ids.all?(Integer)
p ids.uniq.size == ids.size

[
  "break if ready",
  "retry",
  "next",
  "items.each { }; break 1",
  "if ready\n  break\nend",
  "class Report\n  return 1\nend",
  "def build\n  class Report; end\nend",
  "def build\n  Limit = 10\nend",
  "alias $first $1",
  "puts 'a' if a\n\n\nnext 1",
  "1 +",
  "total = price *\n",
  "x = 1\ny = ("
].each do |program|
  RubyVM::AbstractSyntaxTree.parse(program)
rescue SyntaxError => error
  p error.message
end
