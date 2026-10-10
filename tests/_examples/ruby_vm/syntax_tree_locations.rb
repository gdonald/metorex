# RubyVM::AbstractSyntaxTree::Node#locations answers where a node is written,
# then where each keyword and operator MRI records for its type is, with nil
# for one the source does not write.

def span(location)
  return "nil" if location.nil?

  "#{location.first_lineno}:#{location.first_column}-#{location.last_lineno}:#{location.last_column}"
end

def recorded(node, found = [])
  return found unless node.is_a?(RubyVM::AbstractSyntaxTree::Node)

  locations = node.locations
  found << "#{node.type} #{locations.map { |location| span(location) }.join(" ")}" if locations.size > 1
  node.children.each { |child| recorded(child, found) }
  found
end

SOURCES = [
  "alias foo bar",
  "alias $new $old",
  "a && b and c",
  "a || b or c",
  "foo(*rest, &block)",
  "loop { break 1; next 2; redo }",
  "def m; return 1; end",
  "case a\nwhen 1 then :one\nwhen 2\n  :two\nend",
  "case a\nin [x] then x\nin Integer => n if n > 0\n  n\nend",
  "case\nwhen a then 1\nend",
  "value in [1]",
  "value => {name:}",
  "class Foo < Bar\nend",
  "class Foo::Bar; end",
  "class << self; end",
  "module Outer::Inner\nend",
  "Outer::Inner",
  "::Top",
  "defined? x",
  "1..2",
  "1...2",
  "if a..b then end",
  '"a#{b}c"',
  "for item in list do item end",
  "for item in list\n  item\nend",
  "-> (x) { x }",
  "-> x do x end",
  "if a then b elsif c then d else e end",
  "if a # note\n  b\nend",
  "if a\nthen b\nend",
  "x if a",
  "a ? b : c",
  "unless a; b; end",
  "list[0] += 1",
  "point.x ||= 0",
  "END { puts 1 }",
  "/abc/",
  "//",
  "[*items]",
  "def m(*) = other(*)",
  "def m; super(1); super 2; yield(3); yield 4; end",
  "undef foo, bar",
  "while a do b end",
  "begin; b; end while a",
  "until a\n  b\nend",
  "x until a"
]

SOURCES.each do |source|
  puts source.inspect
  recorded(RubyVM::AbstractSyntaxTree.parse(source)).each { |line| puts "  #{line}" }
end

first = RubyVM::AbstractSyntaxTree.parse("if a then b end").children[2].locations.first
p first
p [first.first_lineno, first.first_column, first.last_lineno, first.last_column]
p RubyVM::AbstractSyntaxTree.parse("a").locations.size
