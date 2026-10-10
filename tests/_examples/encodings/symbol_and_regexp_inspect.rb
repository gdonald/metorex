# A Symbol or a Regexp made from text in an encoding other than UTF-8 is
# inspected the way the String would be: a character past ASCII is written
# by the byte or bytes that spell it, and the Regexp keeps those bytes as
# its source. UTF-8 that spells nothing is refused as a pattern.
samples = {
  "binary high" => "\xE3\x81\x82".b,
  "binary ascii" => "abc".b,
  "sjis" => "\x82\xA0".force_encoding("Shift_JIS"),
  "eucjp" => "\xA4\xA2".force_encoding("EUC-JP"),
  "latin1" => "caf\xE9".force_encoding("ISO-8859-1"),
  "utf16le" => "a".encode("UTF-16LE"),
  "invalid utf8" => "\xFF".force_encoding("UTF-8"),
  "utf8 multi" => "あ",
  "binary space" => "a b".b,
  "sjis ascii" => "abc".force_encoding("Shift_JIS"),
}
samples.each do |label, text|
  symbol = (text.to_sym rescue $!)
  regexp = (Regexp.new(text) rescue $!)
  puts "#{label}: #{symbol.inspect} #{symbol.is_a?(Symbol) ? symbol.encoding : "-"} | #{regexp.inspect} #{regexp.is_a?(Regexp) ? regexp.encoding : "-"} | #{(regexp.source.inspect rescue "-")}"
end
p("caf\xE9".force_encoding("ISO-8859-1"))
p(Regexp.new("\xE3\x81\x82".b.force_encoding("UTF-8")) =~ "x\u3042")
p(Regexp.new("ab/c".b))
