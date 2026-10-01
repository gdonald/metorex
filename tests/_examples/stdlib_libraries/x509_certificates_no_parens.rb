# A certificate authority signs its own certificate and then one for a
# server. A store that trusts the authority accepts the server's
# certificate while both are within their validity periods.
require "openssl"

def certificate(subject, issuer, key, signer, serial, from, to, authority)
  made = OpenSSL::X509::Certificate.new
  made.version = 2
  made.serial = serial
  made.subject = OpenSSL::X509::Name.parse(subject)
  made.issuer = issuer || made.subject
  made.public_key = key.public_key
  made.not_before = from
  made.not_after = to
  factory = OpenSSL::X509::ExtensionFactory.new
  factory.subject_certificate = made
  factory.issuer_certificate = authority || made
  usage = authority ? "digitalSignature" : "keyCertSign, cRLSign"
  made.add_extension(factory.create_extension("basicConstraints", "CA:TRUE", true)) unless authority
  made.add_extension(factory.create_extension("keyUsage", usage, true))
  made.add_extension(factory.create_extension("subjectKeyIdentifier", "hash", false))
  made.sign(signer, OpenSSL::Digest.new("SHA256"))
end

now = Time.now
root_key = OpenSSL::PKey::RSA.new(1024)
root = certificate("/CN=Example Root", nil, root_key, root_key, 1, now - 60, now + 3600, nil)
server_key = OpenSSL::PKey::RSA.new(1024)
server = certificate("/CN=server.example", root.subject, server_key, root_key, 2, now - 60, now + 3600, root)

p [root.signature_algorithm, root.serial.to_i, server.issuer.to_s]
p root.extensions.map(&:oid)
p server.extensions[0].to_s
p [server.verify(root_key), server.verify(server_key)]

store = OpenSSL::X509::Store.new
p [store.verify(server), store.error_string]
store.add_cert(root)
p [store.verify(server), store.error_string, store.chain.size]

expired = certificate("/CN=old.example", root.subject, server_key, root_key, 3, now - 60, now - 30, root)
p [store.verify(expired), store.error, store.error_string]
