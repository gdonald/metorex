# Digest::RMD160 gives RIPEMD-160 digests, twenty bytes long, through the same
# methods every Digest class answers.
require("digest/rmd160")
p(Digest::RMD160.hexdigest(""))
p(Digest::RMD160.hexdigest("abc"))
p(Digest::RMD160.hexdigest("a" * 1000))
p(Digest::RMD160.base64digest("message digest"))
digest = Digest::RMD160.new
digest.<<("ab")
digest.update("c")
p(digest.hexdigest, digest.digest_length, digest.block_length, digest.to_s)
p(digest.digest.bytesize)
p(Digest::RMD160.digest("x").encoding)
p(Digest::RMD160.new.hexdigest("abc"))
p(Digest::RMD160.superclass)
