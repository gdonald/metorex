# A pattern gives back what it read when a piece further on cannot carry on,
# which is what lets a group's text be matched again later.
repeated = /(ab)\1/
p repeated.match("abab")[0]
named = /(?<held>ab)\k<held>/
p named.match("abab")[0]

# Text before the match can be checked without reading it.
p "foobar"[/(?<=foo)bar/]
p "bazbar"[/(?<!foo)bar/]

# A group that gives nothing back once it has matched.
atomic = /(?>a*)a/
p atomic.match("aaa")
possessive = /a*+a/
p possessive.match("aaa")
greedy = /a*a/
p greedy.match("aaa")[0]

# The same name may be written on more than one group, and the one that
# matched is the one it stands for.
sides = /(?<side>left)|(?<side>right)/
p sides.match("right")[:side]

# A group's pattern can be matched again by name.
called = /(?<pair>a)\g<pair>/
p called.match("aa")[0]

# The runs a backslash names cover ASCII, while the bracket names cover every
# character they fit.
p "é" =~ /\w/
p "é" =~ /[[:alpha:]]/

# One character as a reader sees it, marks and all.
grapheme = /\X/
p grapheme.match("\u{1F918}\u{1F3FD}")[0]
