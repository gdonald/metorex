# A string carries the encoding it says it is in, and two strings holding the
# same text are still two strings.
require "base64"
require "securerandom"

p "hello".encoding
held = "hello"
held.force_encoding("EUC-JP")
p held.encoding
p held[0, 3].encoding
p held.upcase.encoding
p "plain"[0, 3].encoding

p "hello".b.encoding
p "hello".b == "hello"
p [65, 66].pack("C*").encoding
p [].pack("").encoding
p "abc".encode("US-ASCII").encoding
p Encoding::BINARY.equal?(Encoding::ASCII_8BIT)

# Identity, which the encoding tag rides along with.
first = "same"
second = "same"
p first == second
p first.equal?(second)
p first.equal?(first)
p first.object_id == second.object_id
p first.object_id == first.object_id

# The values that write themselves as one unchanging string.
p nil.to_s.equal?(nil.to_s)
p true.to_s.equal?(true.to_s)
p false.to_s.equal?(false.to_s)
p :held.name.equal?(:held.name)
module Named; end
p Named.name.equal?(Named.name)
p Named.name

# A string cut from a match carries the subject's encoding.
subject = "one two"
subject.force_encoding("EUC-JP")
found = subject.match(/two/)
p found.pre_match.encoding

p Base64.encode64("hi").encoding
p Base64.decode64("aGk=").encoding
p SecureRandom.bytes(4).encoding
p Time.utc(2001, 1, 1).to_s.encoding
