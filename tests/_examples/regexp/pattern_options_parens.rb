# The options a pattern carries read back as a bitvector. A pattern written
# with an encoding after it matches in that encoding whatever the text is
# tagged with, and one written with `n` matches bytes.
plain = //
ignoring = /cat/i
spaced = /cat/x
lines = /cat/m
several = /cat/mix
p(plain.options)
p(ignoring.options & Regexp::IGNORECASE)
p(spaced.options & Regexp::EXTENDED)
p(lines.options & Regexp::MULTILINE)
p(several.options)

utf8 = //u
euc = //e
sjis = //s
bytes = //n
p(utf8.options & Regexp::FIXEDENCODING)
p(euc.options & Regexp::FIXEDENCODING)
p(sjis.options & Regexp::FIXEDENCODING)
p(bytes.options & Regexp::FIXEDENCODING)
p(bytes.options & Regexp::NOENCODING)

begin
  Regexp.allocate.options
rescue TypeError => trouble
  p(trouble.message)
end
