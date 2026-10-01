# An X.509 distinguished name: a list of attributes, each with the ASN.1
# string type its value is written in. It is written in the forms OpenSSL
# prints, each escaping a different set of characters.
require "openssl"

name = OpenSSL::X509::Name.parse("/DC=org/DC=ruby-lang/CN=www.ruby-lang.org")
p name.to_s
p name.to_a
p name.to_s(OpenSSL::X509::Name::RFC2253)
p name.to_s(OpenSSL::X509::Name::ONELINE)
p name.inspect
p OpenSSL::X509::Name.parse("C=US, O=Acme, commonName=x").to_a

special = OpenSSL::X509::Name.new([["CN", "a+b"], ["O", "x,y"]])
p special.to_s
p special.to_s(OpenSSL::X509::Name::RFC2253)
p special.to_s(OpenSSL::X509::Name::ONELINE)
p special.to_s(OpenSSL::X509::Name::MULTILINE)

p OpenSSL::X509::Name.parse("/CN=A") == OpenSSL::X509::Name.parse("/CN=a")
p OpenSSL::X509::Name.parse("/CN=a") <=> OpenSSL::X509::Name.parse("/O=a")
p OpenSSL::X509::Name.parse("/C=US/CN=a").to_der.unpack1("H*")

["hello", "hello=goodbye"].each do |written|
  begin
    OpenSSL::X509::Name.parse(written)
  rescue TypeError, OpenSSL::X509::NameError => error
    p [error.class, error.message]
  end
end
