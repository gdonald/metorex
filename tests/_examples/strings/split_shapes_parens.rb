# `split` cuts a string on whitespace, on a run of text, or on a pattern.
# A count caps the pieces and leaves the rest in the last one, and the groups
# a pattern names are kept alongside the pieces.
p(" now's  the time  ".split(' '))
p(" now's  the time  ".split(' ', 3))
p(" now's  the time  ".split(' ', -1))
p("1,2,,3,4,,".split(","))
p("1,2,,3,4,,".split(",", -1))
p("1.2.3.4".split(".", 2))
p("hello".split(""))
p("hello".split("", 2))

# A pattern keeps what its groups matched.
p("hello".split(/(el)/))
p("hi!".split(/()/))
p("hello".split(/(el)|(xx)/))
p("aBaBa".split(/(B)()()/, 2))
p("hello".split(//, -1))
p("AABCCBAA".split(/(?=B)/))
p("  a  b  c\nd  ".split(/\s+/))

# With a block each piece is handed over and the string itself is the answer.
collected = []
answered = "chunky-bacon".split("-") { |piece| collected << piece.capitalize }
p(collected)
p(answered)
