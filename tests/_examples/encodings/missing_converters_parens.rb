# Some encodings have no converter to or from anything else. Text that is all
# ASCII reads the same in any ASCII-compatible one, so it is relabeled without
# a converter, and anything else is refused.
p("\x79".b.encode(Encoding::Emacs_Mule).encoding)
p("abc".encode("EUC-TW").encoding)

def refusal()
  yield()
rescue Encoding::ConverterNotFoundError => problem
  problem.message()
end

p(refusal() { "\x80".b().encode(Encoding::Emacs_Mule) })
p(refusal() { "café".encode("EUC-TW") })
p(refusal() { Encoding::Converter.new(Encoding::Emacs_Mule, Encoding::BINARY) })

# A dummy encoding has no converter even for ASCII text, and a name that is no
# encoding at all is refused the same way.
p(refusal() { "abc".encode("UTF-7") })
p(refusal() { "abc".encode("xyz") })

# With no encoding named, `encode` writes the string in the default internal
# encoding, or answers a copy when there is none.
held = [0xA4, 0xA2].pack("C*").force_encoding(Encoding::EUC_JP)
p(held.encode().encoding())
p(held.encode().equal?(held))
Encoding.default_internal = Encoding::UTF_8
p(held.encode())
p(held.encode().encoding())
Encoding.default_internal = Encoding::Emacs_Mule
p("\x79".b().encode().encoding())
p(refusal() { "\x80".b().encode() })
Encoding.default_internal = nil
