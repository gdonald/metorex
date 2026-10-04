# A C extension finding encodings by name and index, tagging objects with
# them, and reading the characters a run of bytes holds in one.
require "tmpdir"
require_relative "build_helper"

class Text < String; end

directory = Dir.mktmpdir
require build_extension("c_encodings.c", "c_encodings", directory)
encodings = CEncodings.new

p([encodings.find("utf-8"), encodings.find("no-such-encoding")])
p([encodings.find_index("UTF-8"), encodings.find_index("no-such-encoding")])
p([encodings.from_index(0), encodings.from_index(2), encodings.from_index(100_000), encodings.from_index(-1)])
p([encodings.to_index(Encoding::US_ASCII), encodings.to_index(nil)])
p([encodings.to_encoding_index(Encoding::UTF_8), encodings.to_encoding_index("ascii"), encodings.to_encoding_index("no-such-encoding")])
p encodings.to_encoding_index(Encoding::UTF_16) == Encoding.list.index(Encoding::UTF_16)
p encodings.indexes.first(3)
p(encodings.indexes.last(2).all? { |index| index >= 0 })
Encoding.default_internal = nil
p encodings.settings.values_at(2, 3)
Encoding.default_internal = "US-ASCII"
p encodings.settings[2]
Encoding.default_internal = nil

p encodings.alias("METOREX-ALIAS", "UTF-8") == Encoding.list.index(Encoding::UTF_8)
p([Encoding.find("metorex-alias"), Encoding.aliases["METOREX-ALIAS"]])
p encodings.alias "METOREX-NOTHING", "no-such-encoding"
dummy_index = encodings.define_dummy("METOREX-DUMMY")
p([dummy_index == Encoding.list.size - 1, Encoding.find("metorex-dummy").dummy?, Encoding.find("METOREX-DUMMY").inspect])
report { encodings.define_dummy "METOREX-DUMMY" }

p([encodings.get("text"), encodings.get(Text.new("sub")), encodings.get(/regexp/), encodings.get(Encoding::BINARY), encodings.get(5)])
p([encodings.get_index("text") == encodings.find_index("UTF-8"), encodings.get_index(nil), encodings.get_index(Object.new)])
binary = encodings.find_index("BINARY")
p encodings.set_index(+"text", binary).encoding
p encodings.set_index(Text.new("sub"), binary).encoding
p encodings.set_index(/regexp/.dup, binary).encoding
report { encodings.set_index Object.new, binary }
report { encodings.associate_index :symbol, binary }
p encodings.associate(+"text", nil).encoding
p encodings.associate(+"text", "US-ASCII").encoding
p encodings.copy(+"text", "ascii".encode("US-ASCII")).encoding
p encodings.obj_encoding "text".b
p([encodings.compatible("abc".encode("US-ASCII"), "é"), encodings.compatible("\xff".b, "é")])
p encodings.check("abc".encode("US-ASCII"), "é")
report { encodings.check "\xff".b, "é" }

p([encodings.str_new("\xee", "US-ASCII").encoding, encodings.str_new("ab", nil).encoding])
p([encodings.str_new_cstr("US-ASCII"), encodings.str_new_cstr("US-ASCII").encoding, encodings.str_new_static.encoding])
p(["abc".b, "\xee".b, "abc", "é", "\xee".dup.force_encoding("UTF-8"), "\xee".dup.force_encoding("US-ASCII")].map { |text| encodings.coderange(text) })
p([encodings.ascii_only("abc"), encodings.ascii_only("é")])

p([0x24, 0xA2, 0x20AC, 0x24B62].map { |code| encodings.codelen(code, "UTF-8") })
p([encodings.mbcput(0x20AC, "UTF-8"), encodings.mbcput(0x24, "UTF-16BE").bytes, encodings.mbcput(0x24B62, "UTF-16LE").bytes, encodings.mbcput(0xE9, "ISO-8859-1").bytes])
word = "こにちわ"
p([encodings.str_length(word, 12), encodings.str_length(word, 5), encodings.str_length(word.dup.force_encoding("UTF-16BE"), 12)])
p([encodings.to_codepoint("é"), encodings.to_codepoint(""), encodings.to_codepoint("\xC3".dup.force_encoding("UTF-8"))])
cases = [
  "hello", "", "é", "\xC3".dup.force_encoding("UTF-8"), "\xE3\x81".dup.force_encoding("UTF-8"), "\xFF".dup.force_encoding("UTF-8"),
  "\xE3A".dup.force_encoding("UTF-8"), "\x00".dup.force_encoding("UTF-16BE"), "\xD8\x00\xDC".dup.force_encoding("UTF-16BE"),
  "\x41".dup.force_encoding("UTF-16LE"), "\xA4".dup.force_encoding("EUC-JP"), "\xEE".dup.force_encoding("US-ASCII"),
  "\xC3".dup.force_encoding("UTF8-MAC")
]
cases.each { |text| p(encodings.precise_length(text)) }
p([encodings.nth("hüllo", 3), encodings.nth("hüllo", 9), encodings.nth("hüllo", -1)])
p([encodings.codepoint_length("$"), encodings.codepoint_length("€"), encodings.codepoint_length("\x00\x41".dup.force_encoding("UTF-16BE"))])
report { encodings.codepoint_length "" }
report { encodings.codepoint_length("\xA0\xA1".dup.force_encoding("UTF-8")) }
p([encodings.left_head("éééé", 7), encodings.left_head("é", 1), encodings.left_head("a", 1), encodings.left_head("a".b, 88)])
p([encodings.classes("a".ord, "US-ASCII"), encodings.classes(" ".ord, "UTF-8"), encodings.classes(0xE9, "ISO-8859-1"), encodings.classes(0xE9, "US-ASCII")])
p([1, 0x80, 0x800, 0x10000, 0x200000, 0x4000000].map { |code| encodings.uv_to_utf8(code).bytes })
report { encodings.uv_to_utf8 0x80000000 }
begin
  encodings.enc_raise("UTF-8", ArgumentError, "\x81".b)
rescue ArgumentError => error
  p([error.message.encoding, error.message.valid_encoding?, error.message.bytes])
end
p([encodings.case_fold("Upper"), encodings.case_fold("É"), encodings.case_fold(""), encodings.case_fold("$".encode("UTF-16BE")).then { |text, used| [text.bytes, used] }])
p(["UTF-8", "UTF8-DoCoMo", "UTF-16LE", "UTF-16", "US-ASCII", "ISO-8859-1"].map { |name| encodings.is_unicode(name) })

added = 0
begin
  loop { encodings.define_dummy("METOREX-FILLER-#{added += 1}") }
rescue EncodingError => error
  p([error.class, error.message, Encoding.list.size])
end

FileUtils.rm_rf directory
