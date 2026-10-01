# encoding: big5
# A literal in a source written in Big5 holds the bytes it was written
# with, which are not UTF-8.

word = "你好"
p(word.bytes)
p(word.encoding)
p(__ENCODING__)
