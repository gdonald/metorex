# RubyVM::AbstractSyntaxTree parses a program into the nodes MRI's parser
# makes: a SCOPE holding the program's statements, a BLOCK when there is more
# than one, and a node per literal, each spanning the text it was written as.

def dump(node)
  case node
  when RubyVM::AbstractSyntaxTree::Node
    inner = node.children.map { |child| dump(child) }.join(", ")
    "#{node.type}@#{node.first_lineno}:#{node.first_column}-#{node.last_lineno}:#{node.last_column}(#{inner})"
  when Array
    "[#{node.map { |item| dump(item) }.join(", ")}]"
  else
    node.inspect
  end
end

[
  "1", "1.5", "2r", "3i", "-5", "0x1f", "\"text\"", "'single'", "\"a\" \"b\"",
  ":sym", ":\"quoted sym\"", "nil", "true", "false", "self", "__LINE__",
  "1\n2", "1; 2; 3", "", "# comment\n1", "(1)", "1;", "\"line one\nline two\""
].each { |source| puts dump(RubyVM::AbstractSyntaxTree.parse(source)) }

tree = RubyVM::AbstractSyntaxTree.parse "1 ;"
p tree
p [tree.type, tree.children.size]

begin
  RubyVM::AbstractSyntaxTree.parse "1 +"
rescue SyntaxError => trouble
  p trouble.class
end

require "tempfile"
Tempfile.create(["written", ".rb"]) do |file|
  file.write "true\n"
  file.flush
  puts dump(RubyVM::AbstractSyntaxTree.parse_file(file.path))
end
