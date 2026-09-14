# encoding: us-ascii
# `__ENCODING__` names the encoding of the source it is written in, which is
# settled where it is written rather than where the code around it is run.
p __ENCODING__
p eval("__ENCODING__".dup.force_encoding("BINARY"))
p eval("# encoding: utf-8\n__ENCODING__")

# An encoding is an instance of Encoding rather than a class of its own.
p Encoding::UTF_8.is_a?(Encoding)
p Encoding::UTF_8.is_a?(Class)

# `String.new` takes the characters it was given in the encoding the
# `encoding:` keyword names, empty and reading as bytes with no argument at
# all, and refuses anything that does not read as a String.
p String.new.encoding
p String.new("abc", encoding: "EUC-JP").encoding
p String.new("abc", capacity: 100)
begin
  String.new(5)
rescue TypeError => refused
  p refused.class
end
