# `xml: :text` escapes the characters XML content cannot hold as they are,
# and writes each character the destination cannot spell as a hexadecimal
# character reference. `xml: :attr` also escapes the double quote and quotes
# the whole value.
p("<a & b>".encode("UTF-8", xml: :text))
p("say \"hi\"".encode("UTF-8", xml: :text))
p("<ü>".encode(xml: :text))
p("<ü>\"".encode("US-ASCII", xml: :attr))
p("\u{1F600}&".encode("ISO-8859-1", xml: :text))
p("<".encode("UTF-16LE", xml: :attr).bytes())

# A character reference is written ahead of any other replacement.
p("ü".encode("US-ASCII", xml: :text, undef: :replace))
p("ü".encode("US-ASCII", xml: :text, fallback: { "ü" => "u" }))

# Anything but `:text` or `:attr` is refused.
def refusal()
  yield()
rescue ArgumentError => problem
  problem.message()
end

p(refusal() { "".encode("UTF-8", xml: :other) })
p(refusal() { "".encode("UTF-8", xml: "text") })
