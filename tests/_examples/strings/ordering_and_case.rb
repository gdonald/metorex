# A String orders against anything that reads as one, then against anything
# that orders itself, and has no order at all with anything else.
class Spelled < String
end
p("hello" <=> Spelled.new("hello"))

class Reads
  def to_str
    "aaa"
  end
end
p("abc" <=> Reads.new)

class Orders
  def <=>(other)
    -1
  end
end
p("abc" <=> Orders.new)
p("abc" <=> 7)

# Two strings whose bytes are the same but whose encodings are not are ordered
# by where those encodings sit in the list Ruby keeps.
one = [0xFF].pack("C").force_encoding("utf-8")
two = [0xFF].pack("C").force_encoding("iso-8859-1")
p(one <=> two)
p(two <=> one)

# A Symbol compares its case only against another Symbol.
p(:aBcDeF.casecmp(:abcdef))
p(:abc.casecmp("abc"))
p(:abcdef.casecmp?(:ABCDEF))

# Only an encoding that spells the whole of Unicode maps a letter outside
# ASCII onto another case of itself.
p(:"Ã".casecmp?(:"ã"))
p("\xC3".b.to_sym.casecmp?("\xE3".b.to_sym))
