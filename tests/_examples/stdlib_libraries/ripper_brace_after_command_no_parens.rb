# A `{` after a call written with arguments and no parentheses opens no
# block, so the grammar refuses the program and Ripper.sexp answers nil.
# A call the `{` can belong to keeps it.
require "ripper"

[
  "foo 1 { }",
  "p(foo 1 { })",
  "a.b 1 { }",
  "super 1 { }",
  "foo(1) { }",
  "foo (1) { }",
  "foo a { }",
  "a.b c { }",
  "super(1) { }"
].each { |source| puts "#{source.ljust 14} #{(Ripper.sexp source).nil? ? "refused" : "read"}" }
