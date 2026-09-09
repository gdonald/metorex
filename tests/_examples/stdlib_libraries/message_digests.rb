# Message digests. Each algorithm answers the same digest every other
# implementation does, which is the whole point of naming one.

require 'digest'

p Digest::MD5.hexdigest "abc"
p Digest::SHA1.hexdigest "abc"
p Digest::SHA256.hexdigest "abc"
p Digest::SHA384.hexdigest "abc"
p Digest::SHA512.hexdigest "abc"

# The empty message has a digest of its own.
p Digest::SHA256.hexdigest ""

# A digest object takes its message a piece at a time.
running = Digest::SHA256.new
running << "a"
running << "b"
running << "c"
p running.hexdigest
p running.digest_length
p running.block_length

# Asking for the digest of a given message leaves the object blank again.
p running.hexdigest("abc") == Digest::SHA256.hexdigest("abc")
p running.hexdigest == Digest::SHA256.hexdigest("")

# A digest compares equal to the text of its own hexdigest.
p Digest::MD5.new == "d41d8cd98f00b204e9800998ecf8427e"
p Digest::MD5.new.inspect

# SHA2 picks one of the three by bit length.
p Digest::SHA2.hexdigest("abc", 384) == Digest::SHA384.hexdigest("abc")

# Base64 and hex are two ways of naming the same bytes.
p Digest::SHA256.base64digest ""
p Digest.hexencode "sample string"

# Bubble Babble names a message in syllables.
p Digest.bubblebabble ""
p Digest.bubblebabble "foo"
p Digest.bubblebabble "1234567890"
