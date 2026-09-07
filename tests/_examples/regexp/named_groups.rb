# A pattern may write the same group name more than once, and the match
# reports the farthest group under that name that matched.
data = /(?<a>.)(.)(?<a>\d+)(\d)/.match("THX1138.")

p data["a"]
p data.begin("a")
p data.end("a")
p data.names
p data.named_captures

# A name spelled with multi-byte characters counts in characters.
wide = /(?<æ>.)(.)(?<b>\d+)(\d)/.match("THX1138.")
p wide["æ"]
p wide.begin("æ")
p wide.names

# The pattern reports which group numbers each name was written on.
p(/(?<a>.)(?<b>.)(?<a>.)/.names)
p(/(?<a>.)(?<b>.)(?<a>.)/.named_captures)

# Two patterns differing only in the /n encoding option are the same pattern.
p(/x/n == /x/)
p(/x/n.hash == /x/.hash)
p(/x/i == /x/)

# sub and gsub record the match, whether the pattern is a Regexp or a String.
"he[[o".gsub("[", "]")
p $~.regexp
p $~[0]

"hello world".sub(/o (w)/, "O \\1")
p $~[0]
p $1

# A Proc answers itself for to_proc, and reports the methods it has.
doubler = proc { |value| value * 2 }
p doubler.to_proc.equal?(doubler)
p Proc.public_instance_methods(false).include?(:curry)

# A String answers itself for to_str, and its code points one at a time.
p "hi".to_str
p "hi".codepoints
p "héllo".codepoints.first(2)

# A Rational parsed from text ignores underscores between digits.
p "190_22".to_r
p "-190_22.7".to_r

# Summing floats keeps an infinity rather than turning it into a NaN.
p [1.0, Float::INFINITY].sum
p [1.0, -Float::INFINITY].sum
p [2.78, 5.0, 2.5, 4.44, 3.89, 3.89, 4.44, 7.78, 5.0, 2.78, 5.0, 2.5].sum
