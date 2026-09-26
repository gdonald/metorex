# `invalid: :replace` stands a replacement in for each run of bytes the source
# does not spell, and `undef: :replace` for each character the destination
# cannot spell. The replacement is U+FFFD for a Unicode destination, a
# question mark for any other, or what `replace:` names.
broken = "ab\xFFc"
p(broken.encode("ISO-8859-1", invalid: :replace))
p("\xE3\x81\x93\xE3\x81".encode("UTF-8", invalid: :replace))
p("ち\xE3\x81\xFF".encode("UTF-16LE", invalid: :replace).encode("UTF-8"))
p("ち\xE3\x81\xFF".encode("UTF-16LE", invalid: :replace, replace: "foo").encode("UTF-8"))
p("B�".encode(Encoding::US_ASCII, undef: :replace))
p("B�".encode(Encoding::US_ASCII, undef: :replace, replace: "foo"))
p("あ?あ".encode(Encoding::EUC_JP, undef: :replace).bytes())

# Bytes read as another encoding than the one the string is tagged with.
p("あ?あ".b().encode("euc-jp", "utf-8", undef: :replace).bytes())
p(broken.b().encode("iso-8859-1", "utf-8", invalid: :replace))

# Without the option, either kind of trouble is refused.
def refusal()
  yield()
rescue EncodingError => problem
  [problem.class(), problem.message()]
end

p(refusal() { broken.encode("ISO-8859-1") })
p(refusal() { "a\xE3\x81".encode("UTF-16LE") })
p(refusal() { "a\xE3A".encode("UTF-16LE") })
p(refusal() { "B�".encode(Encoding::US_ASCII) })

# Between two names for the same encoding the bytes are copied, and scrubbed
# when `invalid: :replace` asks for that.
held = "あ".b()
copied = held.encode("utf-8", "utf-8")
p(copied)
p(copied.encoding())
p(copied.equal?(held))
p([0x80].pack("C").force_encoding("Emacs-Mule").encode(invalid: :replace))

# A binary byte past ASCII spells no character anywhere else.
p(refusal() { "a\xC3".b().encode("UTF-8") })
p(refusal() { "a\xC3".b().encode("UTF-16LE") })
p("a\xC3".b().encode("UTF-16LE", undef: :replace).encode("UTF-8"))
