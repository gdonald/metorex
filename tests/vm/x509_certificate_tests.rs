// RSA keys, the certificates they sign, and a store deciding whether to
// trust one.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

const CERTIFICATE_HELPER: &str = "require 'openssl'
def certificate(subject, issuer, key, signer, from, to)
  made = OpenSSL::X509::Certificate.new
  made.version = 2
  made.serial = 7
  made.subject = OpenSSL::X509::Name.parse(subject)
  made.issuer = issuer || made.subject
  made.public_key = key.public_key
  made.not_before = from
  made.not_after = to
  made.sign(signer, 'SHA256')
end
";

fn inspected(code: &str) -> String {
    let source = format!(
        "{CERTIFICATE_HELPER}answer = begin\n{code}\nrescue StandardError => error\n  [error.class, error.message]\nend\nanswer.inspect"
    );
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let tokens = Lexer::new(&source).tokenize();
            let statements = Parser::new(tokens).parse().expect("parse failed");
            let mut vm = VirtualMachine::new();
            match vm.execute_program(&statements).expect("execution failed") {
                Some(Object::String(text)) => text.as_str().to_string(),
                other => panic!("expected an inspection, got {other:?}"),
            }
        })
        .expect("thread failed")
        .join()
        .expect("thread panicked")
}

#[test]
fn a_generated_key_has_a_modulus_of_the_size_asked_for() {
    assert_eq!(
        inspected(
            "key = OpenSSL::PKey::RSA.new(512)\n[key.n.num_bits, key.e.to_i, key.private?, key.public_key.private?, key.n.to_i == key.p.to_i * key.q.to_i]"
        ),
        "[512, 65537, true, false, true]"
    );
}

#[test]
fn a_signature_verifies_only_for_the_data_signed() {
    assert_eq!(
        inspected(
            "key = OpenSSL::PKey::RSA.new(512)\nsignature = key.sign('SHA256', 'data')\npublic = key.public_key\n[signature.bytesize, public.verify('SHA256', signature, 'data'), public.verify('SHA256', signature, 'other'), public.verify('SHA256', 'short', 'data'), key.sign(OpenSSL::Digest.new('SHA256'), 'data') == signature]"
        ),
        "[64, true, false, false, true]"
    );
}

#[test]
fn a_public_key_cannot_sign() {
    assert_eq!(
        inspected("OpenSSL::PKey::RSA.new(512).public_key.sign('SHA256', 'data')"),
        "[ArgumentError, \"private key is needed\"]"
    );
}

#[test]
fn a_key_too_small_for_the_digest_cannot_sign_with_it() {
    assert_eq!(
        inspected("OpenSSL::PKey::RSA.new(512).sign('SHA512', 'data')"),
        "[OpenSSL::PKey::PKeyError, \"EVP_DigestSign: RSA lib\"]"
    );
}

#[test]
fn text_that_holds_no_key_is_refused() {
    assert_eq!(
        inspected(
            "[(OpenSSL::PKey::RSA.new('big') rescue [$!.class, $!.message]), (OpenSSL::PKey::RSA.new(:big) rescue $!.class)]"
        ),
        "[[OpenSSL::PKey::PKeyError, \"Neither PUB key nor PRIV key\"], TypeError]"
    );
}

#[test]
fn a_key_is_read_back_from_der_and_pem() {
    assert_eq!(
        inspected(
            "key = OpenSSL::PKey::RSA.new(512)\nread = ->(text) { OpenSSL::PKey::RSA.new(text) }\n[read[key.to_der].private?, read[key.to_der].to_der == key.to_der, read[key.public_key.to_der].private?, read[key.to_pem].private?, read[key.public_key.to_pem].n == key.n, key.to_pem.lines.first, key.public_key.to_pem.lines.first, key.public_to_pem == key.public_key.to_pem]"
        ),
        "[true, true, false, true, true, \"-----BEGIN RSA PRIVATE KEY-----\\n\", \"-----BEGIN PUBLIC KEY-----\\n\", true]"
    );
}

#[test]
fn a_bare_pkcs1_public_key_is_read() {
    assert_eq!(
        inspected(
            "key = OpenSSL::PKey::RSA.new(512)\nbare = OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__integer__(key.n.to_i), OpenSSL::ASN1.__integer__(key.e.to_i))\npem = \"-----BEGIN RSA PUBLIC KEY-----\\n#{[bare].pack('m48')}-----END RSA PUBLIC KEY-----\\n\"\n[OpenSSL::PKey::RSA.new(bare).n == key.n, OpenSSL::PKey::RSA.new(pem).e.to_i, (OpenSSL::PKey::RSA.new(\"-----BEGIN RSA PUBLIC KEY-----\\nAA==\\n-----END RSA PUBLIC KEY-----\\n\") rescue $!.class)]"
        ),
        "[true, 65537, OpenSSL::PKey::PKeyError]"
    );
}

#[test]
fn a_big_number_answers_as_an_integer() {
    assert_eq!(
        inspected(
            "held = OpenSSL::BN.new(255)\n[held.to_i, held.to_s, held.to_s(16), held == 255, held > OpenSSL::BN.new(1), held.num_bytes, OpenSSL::BN.new(\"\\x01\\x00\".b, 2).to_i, OpenSSL::BN.new('ff', 16).to_i]"
        ),
        "[255, \"255\", \"FF\", true, true, 1, 256, 255]"
    );
}

#[test]
fn extensions_are_written_in_der() {
    assert_eq!(
        inspected(
            "factory = OpenSSL::X509::ExtensionFactory.new\n[factory.create_extension('basicConstraints', 'CA:TRUE', true), factory.create_extension('keyUsage', 'keyCertSign, cRLSign', true), factory.create_extension('keyUsage', 'digitalSignature', true), factory.create_extension('basicConstraints', 'CA:FALSE')].map { |held| held.to_der.unpack1('H*') }"
        ),
        "[\"300f0603551d130101ff040530030101ff\", \"300e0603551d0f0101ff040403020106\", \"300e0603551d0f0101ff040403020780\", \"30090603551d1304023000\"]"
    );
}

#[test]
fn an_extension_shows_its_uses_by_name() {
    assert_eq!(
        inspected(
            "held = OpenSSL::X509::ExtensionFactory.new.create_extension('keyUsage', 'keyCertSign, cRLSign', true)\n[held.oid, held.value, held.critical?, held.to_s, held.to_a]"
        ),
        "[\"keyUsage\", \"Certificate Sign, CRL Sign\", true, \"keyUsage = critical, Certificate Sign, CRL Sign\", [\"keyUsage\", \"Certificate Sign, CRL Sign\", true]]"
    );
}

#[test]
fn an_extension_without_what_it_needs_is_refused() {
    assert_eq!(
        inspected(
            "factory = OpenSSL::X509::ExtensionFactory.new\n[(factory.create_extension('subjectKeyIdentifier', 'hash') rescue $!.message), (factory.create_extension('nope', 'x') rescue $!.message), (factory.create_extension('keyUsage', 'flying') rescue $!.class)]"
        ),
        "[\"subjectKeyIdentifier = hash: error in extension (name=subjectKeyIdentifier, value=hash)\", \"nope = x: error in extension (name=nope, value=x)\", OpenSSL::X509::ExtensionError]"
    );
}

#[test]
fn a_new_certificate_has_nothing_set() {
    assert_eq!(
        inspected(
            "held = OpenSSL::X509::Certificate.new\n[held.version, held.serial.to_i, held.subject.to_s, held.extensions, (held.not_before rescue $!.class), (held.public_key rescue $!.message), (held.signature_algorithm rescue $!.message), (held.to_der rescue $!.message), (held.verify(OpenSSL::PKey::RSA.new(512)) rescue $!.message)]"
        ),
        "[0, 0, \"\", [], OpenSSL::ASN1::ASN1Error, \"decode error\", \"OBJ_obj2txt\", \"illegal zero content\", \"unknown signature algorithm\"]"
    );
}

#[test]
fn a_signed_certificate_verifies_against_its_signer() {
    assert_eq!(
        inspected(
            "key = OpenSSL::PKey::RSA.new(512)\nnow = Time.now\nheld = certificate('/CN=a', nil, key, key, now - 60, now + 60)\n[held.verify(key), held.verify(OpenSSL::PKey::RSA.new(512)), held.signature_algorithm, held.to_der.getbyte(0), held.not_before.to_i == (now - 60).to_i]"
        ),
        "[true, false, \"sha256WithRSAEncryption\", 48, true]"
    );
}

#[test]
fn a_validity_bound_after_2049_is_written_as_generalized_time() {
    assert_eq!(
        inspected(
            "key = OpenSSL::PKey::RSA.new(512)\nheld = certificate('/CN=a', nil, key, key, Time.at(0), Time.at(2_600_000_000))\n[held.to_der.b.include?(\"\\x18\\x0f20520522141320Z\".b), held.to_der.b.include?(\"\\x17\\x0d700101000000Z\".b)]"
        ),
        "[true, true]"
    );
}

#[test]
fn a_store_names_why_it_refuses_a_certificate() {
    assert_eq!(
        inspected(
            "root_key = OpenSSL::PKey::RSA.new(512)\nleaf_key = OpenSSL::PKey::RSA.new(512)\nnow = Time.now\nroot = certificate('/CN=r', nil, root_key, root_key, now - 60, now + 60)\nleaf = certificate('/CN=l', root.subject, leaf_key, root_key, now - 60, now + 60)\nstore = OpenSSL::X509::Store.new\nresults = []\nresults << [store.verify(root), store.error]\nresults << [store.verify(leaf), store.error]\nresults << [store.verify(leaf, [root]), store.error]\nstore.add_cert(root)\nresults << [store.verify(leaf), store.error, store.chain.size]\nfuture = certificate('/CN=f', nil, root_key, root_key, now + 600, now + 900)\nstore.add_cert(future)\nresults << [store.verify(future), store.error, store.error_string]\nresults"
        ),
        "[[false, 18], [false, 20], [false, 19], [true, 0, 2], [false, 9, \"certificate is not yet valid or the system clock is incorrect\"]]"
    );
}

#[test]
fn a_certificate_signed_by_a_different_key_has_no_issuer_in_the_store() {
    assert_eq!(
        inspected(
            "root_key = OpenSSL::PKey::RSA.new(512)\nnow = Time.now\nroot = certificate('/CN=r', nil, root_key, root_key, now - 60, now + 60)\nforged = certificate('/CN=l', root.subject, root_key, OpenSSL::PKey::RSA.new(512), now - 60, now + 60)\nstore = OpenSSL::X509::Store.new\nstore.add_cert(root)\n[store.verify(forged), store.error_string]"
        ),
        "[false, \"unable to get local issuer certificate\"]"
    );
}

#[test]
fn a_key_too_small_or_with_an_unusable_exponent_is_refused() {
    assert_eq!(
        inspected(
            "[(OpenSSL::PKey::RSA.new(100) rescue $!.message), (OpenSSL::PKey::RSA.new(512, 4) rescue $!.message), (OpenSSL::PKey::RSA.new(512, -1) rescue $!.message), OpenSSL::PKey::RSA.new(512, 3).e.to_i]"
        ),
        "[\"EVP_PKEY_CTX_ctrl_str(ctx, \\\"rsa_keygen_bits\\\", \\\"100\\\"): key size too small\", \"EVP_PKEY_keygen: pub exponent out of range\", \"EVP_PKEY_CTX_ctrl_str(ctx, \\\"rsa_keygen_pubexp\\\", \\\"-1\\\"): invalid negative value\", 3]"
    );
}

#[test]
fn the_key_generator_refuses_what_it_cannot_use() {
    assert_eq!(
        inspected(
            "[(OpenSSL.__rsa_generate__(512, 4) rescue $!.message), (OpenSSL.__rsa_generate__ rescue $!.class), (OpenSSL.__rsa_generate__(512, 'x') rescue $!.message)]"
        ),
        "[\"invalid key size or exponent\", ArgumentError, \"invalid exponent\"]"
    );
}

#[test]
fn the_object_table_answers_nothing_for_what_it_does_not_hold() {
    assert_eq!(
        inspected(
            "[OpenSSL.__object__(:encode, 'CN'), OpenSSL.__object__(:find, 5), OpenSSL.__object__(:find, 'no such object'), OpenSSL.__object__(:find, '1.2.3.4')]"
        ),
        "[nil, nil, nil, [\"1.2.3.4\", \"1.2.3.4\", \"1.2.3.4\"]]"
    );
}
