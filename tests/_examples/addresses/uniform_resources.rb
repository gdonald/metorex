# A URI is split into the components RFC 2396 names, and the scheme decides
# which class carries them.
require 'uri'

page = URI.parse("http://user:pass@example.com:8080/path/to/page?query=val#top")

p page.class
p page.scheme
p page.userinfo
p page.user
p page.password
p page.host
p page.port
p page.path
p page.query
p page.fragment
p page.request_uri
p page.to_s
p page.absolute?
p page.select(:scheme, :host, :port)

p URI.parse("https://example.com/").port
p URI.parse("http://example.com/").port
p URI.parse("ftp://anonymous@ruby-lang.org/pub/ruby.tar.gz;type=i").class
p URI.parse("ftp://anonymous@ruby-lang.org/pub/ruby.tar.gz;type=i").path
p URI.parse("ftp://anonymous@ruby-lang.org/pub/ruby.tar.gz;type=i").typecode
p URI.parse("mailto:spam@example.com?subject=Hello").class
p URI.parse("mailto:spam@example.com?subject=Hello").to
p URI.parse("mailto:spam@example.com?subject=Hello").headers
p URI.parse("ldap://ldap.example.com/o=Example,c=US?postalAddress").dn
p URI.parse("news:comp.lang.ruby").opaque

# A relative reference is laid over a base the way RFC 2396 says.
base = URI.parse("http://a/b/c/d;p?q")
p (base + "g").to_s
p (base + "../g").to_s
p (base + "/./g").to_s
p (base + "?y").to_s
p base.route_to("http://a/b/c/g").to_s
p URI.join("http://localhost/a/b/c/d", "../../e/f", "g/h/../i").to_s

p URI.escape("a b&c")
p URI.unescape("a%20b%26c")
p URI.encode_www_form_component("a b&c")
p URI.encode_www_form([["name", "ruby"], ["age", "30"]])
p URI.decode_www_form("name=ruby&age=30")
p URI.extract("see http://example.com/x and mailto:a@b.c today")

p URI("http://example.com") == URI("http://example.com/")
p URI("http://exAMPLE.cOm") == URI("http://example.com")
p URI("http://example.com/paTH") == URI("http://example.com/path")
p URI("http://example.com").normalize.to_s
