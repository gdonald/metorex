# What a pattern says about itself, and the text a pattern is written from.
p(Regexp.escape("\*?{}.+^$[]()- "))
p(Regexp.escape("\n\r\f\t"))
p(Regexp.escape("abc").encoding)
p(Regexp.quote("a.b"))

folded = /(?i:nothing)/
p(folded.to_s)
folding = /abc/i
plain = /abc/
p(folding.to_s)
p(plain.to_s)

terminator = %r@\@@
p(terminator.source)
p(%r+\++.source)

ascii_only = /[\u{20}-\u{7E}]/
wider = /[\u{20}-\u{7EE}]/
p(ascii_only.source.encoding)
p(wider.source.encoding)

p(Regexp.try_convert(/foo/))
p(Regexp.try_convert("foo"))

p(Regexp.linear_time?(/a/))
p(Regexp.linear_time?(/(a)\1/))
p([Regexp::IGNORECASE, Regexp::EXTENDED, Regexp::MULTILINE])

p(Regexp.new("").frozen?)
literal = /ab/
p(literal.frozen?)
p(%s{foo bar})
