# Digests keyed with a secret, keys derived from a password, and escaping
# text for a URL or a page.

require 'openssl'
require 'cgi/escape'

# A digest is named the way OpenSSL names one, in any spelling.
p(OpenSSL::Digest.new("sha1").name)
p(OpenSSL::Digest::SHA256.new.name)
p(OpenSSL::Digest.hexdigest "sha256", "abc")
p(OpenSSL::Digest.new("SHA1").digest_length)
p(OpenSSL::Digest.new("SHA512").block_length)

# A keyed digest says both what the message is and that whoever wrote it
# held the key.
message = "The quick brown fox jumps over the lazy dog"
p(OpenSSL::HMAC.hexdigest OpenSSL::Digest.new("SHA1"), "key", message)

# A key derived from a password takes as long to guess as it does to derive.
settings = {salt: "salt", iterations: 100, length: 16, hash: "sha1"}
derived = OpenSSL::KDF.pbkdf2_hmac "secret", **settings
p(derived.length)
p(derived == OpenSSL::KDF.pbkdf2_hmac("secret", **settings))

# Comparing without letting how long it takes say where two strings differ.
p(OpenSSL.fixed_length_secure_compare "abc", "abc")
p(OpenSSL.fixed_length_secure_compare "abc", "abd")

# A URL carries only a few characters as themselves.
p(CGI.escape "a b&c~")
p(CGI.unescape "a+b%26c")
p(CGI.escapeURIComponent "a b/c")
p(CGI.unescapeURIComponent "a%20b%2Fc")

# A page reads five characters as markup.
p(CGI.escapeHTML %[& < > " '])
p(CGI.unescapeHTML "&amp; &lt; &gt; &quot; &#99;")
p(CGI.escapeElement '<BR><A HREF="url"></A>', "A")
p(CGI.unescapeElement '<BR>&lt;A HREF=&quot;url&quot;&gt;', "A")
