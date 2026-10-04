# A C extension growing, shrinking and writing the bytes of strings,
# making strings in an encoding, and converting, slicing and interning them.
require "tmpdir"
require_relative "build_helper"

class Spelled
  def to_str = "spelled"
end

class Text < String; end

directory = Dir.mktmpdir
require build_extension("c_strings.c", "c_strings", directory)
strings = CStrings.new

text = +"abcdefghij"
strings.modify text
p strings.set_len text, 4
p strings.set_len text, 8
strings.set_len text, 1
strings.poke text, 1, "B".ord
strings.poke text, 2, "C".ord
p strings.set_len text, 3
wide = "abcdefghij".dup.force_encoding("UTF-16BE")
p strings.set_len(wide, 4).b
p strings.peek(wide, 4, 2).bytes
report { strings.set_len(+"short", 100) }
report { strings.set_len(+"short", -1) }

buffer = strings.buf_new(16)
p([buffer, buffer.encoding, strings.capacity(buffer) >= 16])
strings.poke buffer, 0, 195
strings.poke buffer, 1, 169
strings.set_len buffer, 2
p buffer.force_encoding("UTF-8")
grown = +"abc"
p strings.modify_expand(grown, 50) >= 53
p strings.modify_expand(grown, 1) >= 53
p grown
report { strings.modify_expand(+"abc", -1) }
p strings.resize(+"test", 2)
p strings.resize(+"te", 6).bytes
report { strings.resize(+"test", -1) }
report { strings.modify "frozen".freeze }

watched = +"abc"
p(strings.write_then_yield(watched) { |seen| seen.setbyte(1, "Y".ord) })
p watched
p(strings.write_then_yield(+"abc") { |seen| seen.replace("lmnop") })

made, same_pointer = strings.static_pointer
p([made, made.encoding, same_pointer])
made << "!"
p made
literal = strings.usascii_literal
p([literal, literal.encoding])
p(strings.constructors.map { |held| [held, held.encoding.name] }.first(5))
p strings.external_with("\x80abc".b, "US-ASCII").encoding
p strings.external_with("abc", "UTF-8").encoding
Encoding.default_internal = "EUC-JP"
p strings.external_with("\xE3\x81\x82".b, "UTF-8").bytes
Encoding.default_internal = nil
p strings.locale_strings.map(&:encoding).uniq == [Encoding.find("locale")]
hidden, revealed, tmp = strings.tmp_class
p([hidden, revealed, tmp.bytes, tmp.encoding])

source = "copy"
dup, shared, frozen, classed = strings.copies(source)
p([dup.equal?(source), shared.equal?(source), frozen.frozen?, frozen.equal?(source), classed])
already = "frozen".freeze
p strings.copies(already)[2].equal?(already)
p strings.copies(Text.new("sub"))[3].class
p strings.drop_bytes(+"12345678", 4)
p strings.drop_bytes("12345678".encode("UTF-16LE"), 4).encode("UTF-8")
p strings.free_string(+"x")

locked = +"locked"
p strings.lock(locked).equal?(locked)
report { strings.lock locked }
report { locked.upcase! }
report { strings.modify locked }
p strings.unlock(locked).upcase!
report { strings.unlock locked }
report { strings.lock "frozen".freeze }

p([strings.interned("abc", false).encoding, strings.interned("\xC3\xA9".b, false).encoding, strings.interned("abc", false).frozen?])
p strings.interned("abc", false).equal?(strings.interned_cstr("abc", false))
p([strings.interned("abc", "UTF-8").encoding, strings.interned("abc", nil).encoding, strings.interned_cstr("abc", "UTF-8").equal?(-"abc")])
p strings.to_interned(+"abc").equal?(-"abc")

p strings.joins "ab", "cd"
p strings.append(+"ab", Spelled.new)
report { strings.append(+"ab", 5) }
p strings.cats(+"start")
p(strings.enc_cat("hi ".dup.force_encoding("US-ASCII"), "résumé", "UTF-8").then { |held| [held, held.encoding] })
p([strings.compare("a", "b"), strings.compare("b", "a"), strings.compare("a", "a")])
p strings.lengths "hëllo", 3
p([strings.subpos("hëllo", 2, 3), strings.subpos("hello", -2, 9), strings.subpos("hello", 6, 1), strings.subpos("hello", -6, 1), strings.subpos("hello", 1, -1)])
p strings.slices "hëllo"
p strings.update(+"hello", 2, 3, "wuh")
p strings.split "a,b,,c"
p strings.readings(+"word")
p([strings.to_inum("1234a", 10, false), strings.to_inum("ff", 16, true), strings.cstr2inum("10", 16), strings.str2inum(Spelled.new.then { "42" }, 10)])
report { strings.to_inum "1234a", 10, true }
report { strings.cstr2inum "12x", 0 }

invalid = "a\xFFc".dup.force_encoding("UTF-8")
p strings.encode invalid, "US-ASCII", 2, nil
p(strings.encode(invalid, "US-ASCII", 2, { replace: "b" }.freeze))
p strings.encode "é", "US-ASCII", 32, nil
p strings.encode("abc", Encoding::ISO_8859_1, 0, nil).encoding

ascii = "abc".dup.force_encoding("US-ASCII")
p([strings.conv(ascii, "US-ASCII", nil).equal?(ascii), strings.conv(ascii, nil, "US-ASCII").equal?(ascii)])
p([strings.conv(ascii, "UTF-8", "US-ASCII").equal?(ascii), strings.conv(ascii, "UTF-8", "ISO-8859-1").encoding])
p strings.conv("abc".b, "US-ASCII", "ASCII-8BIT").encoding
p strings.conv("\xE3\x81\x82".dup.force_encoding("UTF-8"), "UTF-8", "EUC-JP").bytes
broken = "\xEE".dup.force_encoding("UTF-8")
p strings.conv_opts(broken, "UTF-8", "EUC-JP").equal?(broken)
Encoding.default_external = "ISO-8859-1"
p(strings.exports("Hëllo", "UTF-8").map { |held| held.encoding.name })
Encoding.default_external = "UTF-8"

p([strings.string_of("plain"), strings.string_of(Spelled.new), strings.string_of(5), strings.to_str(Spelled.new)])
report { strings.to_str 5 }
p strings.formatted "Hello"
p strings.vcatf(+"count ")

FileUtils.rm_rf directory
