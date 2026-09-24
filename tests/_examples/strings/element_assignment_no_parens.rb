def refusal
  yield
  "nothing raised"
rescue StandardError => error
  "#{error.class}: #{error.message}"
end

text = "hello"
text[0] = "J"
puts text
text[1, 3] = "ELL"
puts text
text["LL"] = "ll"
puts text
text[/o$/] = "0"
puts text
text[1..2] = "a"
puts text
text[2..] = "rs"
puts text
text[..0] = "B"
puts text

text = "hello"
puts text[0] = "y"

dated = "2024-06-01"
dated[/(\d+)-(\d+)/, 2] = "07"
puts dated
dated[/(?<year>\d+)-/, "year"] = "2025"
puts dated
dated[/(?<day>\d+)$/, :day] = "15"
puts dated
dated[/(\d+)-(\d+)-(\d+)/, -1] = "20"
puts dated

puts refusal { "abc"[/z/] = "x" }
puts refusal { "abc"[/a(b)/, 2] = "x" }
puts refusal { "abc"[/a(b)/, -2] = "x" }
puts refusal { "a b"[/a (b)(Z)?/, 2] = "x" }
puts refusal { "abc"["z"] = "x" }
puts refusal { "abc"[5] = "x" }
puts refusal { "abc"[1, -1] = "x" }
puts refusal { "abc"[-4..-2] = "x" }
puts refusal { "abc"[4..5] = "x" }
puts refusal { "abc"[1] = 7 }
puts refusal { "abc".send :[]=, "x" }

widened = " ".force_encoding Encoding::US_ASCII
widened[0] = [160].pack("C").force_encoding Encoding::BINARY
puts widened.encoding
puts refusal { "あれ"[0] = "が".encode Encoding::EUC_JP }

class Label
  def to_str
    "tag"
  end
end

tagged = "a-b"
tagged[1.9] = Label.new
puts tagged

puts "first", tagged[0] = "z"
puts tagged
