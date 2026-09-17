# A pattern gives back what it read when a piece further on cannot carry on,
# which is what lets a group's text be matched again later.
p(/(ab)\1/.match("abab")[0])
p(/(?<held>ab)\k<held>/.match("abab")[0])

# Text before the match can be checked without reading it.
p("foobar"[/(?<=foo)bar/])
p("bazbar"[/(?<!foo)bar/])

# A group that gives nothing back once it has matched.
p(/(?>a*)a/.match("aaa"))
p(/a*+a/.match("aaa"))
p(/a*a/.match("aaa")[0])

# The same name may be written on more than one group, and the one that
# matched is the one it stands for.
p(/(?<side>left)|(?<side>right)/.match("right")[:side])

# A group's pattern can be matched again by name.
p(/(?<pair>a)\g<pair>/.match("aa")[0])

# The runs a backslash names cover ASCII, while the bracket names cover every
# character they fit.
p("é" =~ /\w/)
p("é" =~ /[[:alpha:]]/)

# One character as a reader sees it, marks and all.
p(/\X/.match("\u{1F918}\u{1F3FD}")[0])
