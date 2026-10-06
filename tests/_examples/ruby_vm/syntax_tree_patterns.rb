# RubyVM::AbstractSyntaxTree names each pattern the way MRI's parser does:
# array, find and hash patterns, alternatives, bindings, pins and guards,
# and the one-line `in` and `=>` matches.

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

SOURCES = [
  "case a; in 1; end",
  "case a; in 1 then b; end",
  "case a; in Integer => x; end",
  "case a; in [1, *r]; end",
  "case a; in [1, 2]; end",
  "case a; in []; end",
  "case a; in {x: 1}; end",
  "case a; in {x:}; end",
  "case a; in {}; end",
  "case a; in {x: 1, **r}; end",
  "case a; in {x: 1, **nil}; end",
  "case a; in [*, 1, *]; end",
  "case a; in 1 | 2; end",
  "case a; in ^b; end",
  "case a; in ^(b + 1); end",
  "case a; in x if x > 1; end",
  "case a; in x unless x; end",
  "case a; in Foo(1, 2); end",
  "case a; in Foo[x:]; end",
  "case a; in 1..; end",
  "case a; in nil; else 2; end",
  "a in Integer",
  "a => x",
  "a => [x, y]",
  "case a; in :s; end",
  "case a; in \"s\"; end",
  "case a; in /r/; end",
  "case a; in -> x { x }; end",
  "case a; in Foo::Bar; end",
  "case a; in String | Symbol => s; end",
  "case a; in x:; end",
  "case a; in 1, 2; end",
  "case a; in 1, *r; end",
  "case a; in x: 1, **r; end",
  "b = 1; case a; in ^b; end",
  "case a; in [x, *]; end",
  "case a; in [*r, 1]; end",
  "case a; in Foo[*, 1, *x]; end",
  "case a; in {x: [1, y]}; end",
  "case a; in Foo(x:); end",
  "case a; in [1] | [2]; end",
  "case a; in {x: 1} => h; end",
  "case a\nin 1\n  b\nin 2\n  c\nend",
  "case a; in 1; b; in 2; c; else; d; end",
  "foo { case a; in x; end }",
  "case a; in @b; end",
  "case a; in ^@b; end",
  "case a; in \"\#{b}\"; end",
  "x = 1; case a; in ^x; end",
  "a in [x]",
  "a => {x:}",
  "case a; in [Integer => x, String]; end",
  "case a; in {name: String => n}; end",
  "case a; in ^$g; end",
  "case a; in ^@@c; end",
  "case a; in [1, *] | []; end",
  "case a; in Foo::Bar(x:); end",
  "case a; in ::Foo[1]; end",
  "foo { |v| case v; in [x]; x; end }",
  "case a\nin [\n  1,\n  2\n]\n  b\nend",
  "case a\nin {x: 1,\n    y: 2}\n  b\nend",
  "case a; in [[1, 2], *rest]; end",
  "case a; in {x: {y: z}}; end",
  "case a; in 1.. | ..0; end",
  "case a; in String if a.size > 1 && b; end",
  "x = 1; case a; in [^x, y]; end",
  "case [1, 2]; in [a, b] then a + b; end",
  "case a; in Integer | Float => n; end",
  "case a; in [*, :x, *post]; end",
  "case a; in nil | true | false; end",
  "case a; in __FILE__; end",
  "case \"x\"; in \"x\" then 1; end",
  "case a; in {x: 1, **}; end",
  "case a; in (1 | 2); end",
  "case a; in **r; end",
  "case a; in Foo(); end",
  "case a; in x, *; end",
  "case a; in 1,; end",
  "case a; in Foo[]; end",
  "case a; in [*, [1], *]; end",
  "case a; in [*r, [1], {x:}]; end",
  "case a; in Foo([1], *); end",
  "case a; in [1], [2]; end",
  "case a; in {x: [1], y: {z:}}; end",
].freeze

SOURCES.each do |source|
  puts(dump(RubyVM::AbstractSyntaxTree.parse(source)))
rescue SyntaxError
  puts("SyntaxError")
end
