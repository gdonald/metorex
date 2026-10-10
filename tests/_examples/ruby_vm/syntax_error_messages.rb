# A program the parser refuses raises SyntaxError naming the token it met
# the way MRI's parser does, and listing the tokens it would have taken when
# there are at most four of them.
SOURCES = [
  "`a` `b`",
  "case a; in @b; end",
  "foo 1 { }",
  "1 +",
  "def f(; end",
  "x = = 1",
  "[1, 2",
  "p(1 2)",
  "class Foo < ; end",
  "foo(a: 1, 2)",
  "{a: 1 b: 2}",
  "1 2",
  "a ? b",
  "def f a, b c; end",
  "-> (x { x }",
  "module 1; end",
  "class foo; end",
  ":",
  "1..2..3",
  "x.y z w",
  "begin; rescue => 1; end",
  "case 1 when 2 then 3 else 4 else 5 end",
  "for in [1]; end",
  "%w(a b",
  "\"abc",
  "[1,,2]",
  "{1 => }",
  "a::B::",
  "-> { } ()",
  "a ||| b",
  "BEGIN 1",
  "def self.; end",
  "0x",
  "1_",
  "def f(*a, *b); end",
  "def f(**a, b); end",
  "x = <<~EOS",
  "when 1",
  "=>",
  "::",
  ";;;)"
].freeze

SOURCES.each do |source|
  RubyVM::AbstractSyntaxTree.parse(source)
  puts("#{source.inspect}: read")
rescue SyntaxError => error
  puts("#{source.inspect}: #{error.message.lines.first.chomp}")
end
