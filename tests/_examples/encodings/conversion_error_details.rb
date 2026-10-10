# The error String#encode raises names the step of the conversion it
# stopped at and carries what MRI's does: the encodings of that step, and
# the character it could not spell or the bytes it could not read, with
# whether the input ended inside a character.
def show(label)
  yield
  puts "#{label}: no error"
rescue EncodingError => e
  attrs = %i[source_encoding_name destination_encoding_name error_char error_bytes readagain_bytes incomplete_input?].filter_map do |name|
    "#{name}=#{e.public_send(name).inspect}" if e.respond_to?(name)
  end
  puts "#{label}: #{e.class}: #{e.message} [#{attrs.join(", ")}]"
end
show("sjis ff") { "\xff".force_encoding("Shift_JIS").encode("UTF-8") }
show("sjis followed") { "\x82\x20".force_encoding("Shift_JIS").encode("UTF-8") }
show("sjis 80") { "a\x80b".force_encoding("Shift_JIS").encode("UTF-8") }
show("euc ff") { "\xff".force_encoding("EUC-JP").encode("UTF-8") }
show("euc followed") { "\xa4\x41".force_encoding("EUC-JP").encode("UTF-8") }
show("euc ss3 short") { "\x8f\xa1".force_encoding("EUC-JP").encode("UTF-8") }
show("euc ss2 bad") { "\x8e\x41".force_encoding("EUC-JP").encode("UTF-8") }
show("utf16 to latin1") { "あ".encode("UTF-16LE").encode("ISO-8859-1") }
show("utf8 to sjis") { "\u{1F600}".encode("Shift_JIS") }
show("utf8 followed") { "\xe3\x41".force_encoding("UTF-8").encode("UTF-16LE") }
show("binary to latin1") { "\xff".b.encode("ISO-8859-1") }
show("sjis to eucjp") { "\x82\xa0".force_encoding("Shift_JIS").encode("EUC-JP") }
show("utf8 to iso2022") { "\u{1F600}".encode("ISO-2022-JP") }
show("sjis to latin1") { "\x82\xA0".force_encoding("Shift_JIS").encode("ISO-8859-1") }
show("invalid utf8") { "\xff".force_encoding("UTF-8").encode("UTF-16LE") }
show("incomplete utf8") { "\xe3\x81".force_encoding("UTF-8").encode("UTF-16LE") }
show("incomplete sjis") { "\x82".force_encoding("Shift_JIS").encode("UTF-8") }
show("incomplete eucjp") { "\xa4\xa2\x8e".force_encoding("EUC-JP").encode("Shift_JIS") }
show("binary to utf8") { "\xff".b.encode("UTF-8") }
show("latin1 to ascii") { "\xff".force_encoding("ISO-8859-1").encode("US-ASCII") }
