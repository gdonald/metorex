# `scan` walks every match in a string, handing the groups to a block as one
# array, and `\G` asks each match to begin where the one before it ended.
p("cruel world".scan(/\w+/))
p("cruel world".scan(/(..)(..)/))
p("hello".scan(//))
p("one two one two".scan(/\G\w+/))
p("one two one two".scan(/\G\w+\s*/))
p("o_o".scan("o"))

collected = []
"a b c".scan(/(\w) (\w)/) { |first, second| collected << [first, second] }
p(collected)

"hello.".scan("l") { "x" }
p($~.begin(0))
"hello.".scan("zz") { "x" }
p($~)
