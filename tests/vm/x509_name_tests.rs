// OpenSSL::X509::Name: building a distinguished name from its attributes,
// the forms it is printed in, and how two names compare.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn inspected(code: &str) -> String {
    let source = format!(
        "require 'openssl'\nanswer = begin\n{code}\nrescue StandardError => error\n  [error.class, error.message]\nend\nanswer.inspect"
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

fn shown(value: &str, format: &str) -> String {
    let format = if format.is_empty() {
        String::new()
    } else {
        format!("(OpenSSL::X509::Name::{format})")
    };
    inspected(&format!(
        "OpenSSL::X509::Name.new([['CN', {value:?}]]).to_s{format}"
    ))
}

#[test]
fn the_default_form_escapes_slashes_and_plus_signs() {
    assert_eq!(shown("a/b+c", ""), "\"/CN=a\\\\/b\\\\+c\"");
}

#[test]
fn the_default_form_writes_bytes_outside_printable_ascii_in_hex() {
    assert_eq!(shown("tab\tx", ""), "\"/CN=tab\\\\x09x\"");
}

#[test]
fn rfc_2253_escapes_separators_and_the_ends_of_a_value() {
    assert_eq!(
        inspected(
            r#"[' lead', 'trail ', '#hash', 'a,b;c<d>e"f\\g'].map { |value| OpenSSL::X509::Name.new([['CN', value]]).to_s(OpenSSL::X509::Name::RFC2253) }"#
        ),
        r#"["CN=\\ lead", "CN=trail\\ ", "CN=\\#hash", "CN=a\\,b\\;c\\<d\\>e\\\"f\\\\g"]"#
    );
}

#[test]
fn the_one_line_form_quotes_a_value_holding_a_separator() {
    assert_eq!(shown("a,b", "ONELINE"), "\"CN = \\\"a,b\\\"\"");
}

#[test]
fn the_compat_form_joins_attributes_with_a_comma_and_space() {
    assert_eq!(
        inspected(
            "OpenSSL::X509::Name.new([['CN', 'a'], ['O', 'b']]).to_s(OpenSSL::X509::Name::COMPAT)"
        ),
        "\"CN=a, O=b\""
    );
}

#[test]
fn the_utf8_form_keeps_characters_beyond_ascii() {
    assert_eq!(
        inspected(r#"OpenSSL::X509::Name.new([['CN', "caf\u00e9"]]).to_utf8"#),
        "\"CN=caf\u{e9}\""
    );
}

#[test]
fn a_long_name_is_a_short_name_in_the_attribute_list() {
    assert_eq!(
        inspected("OpenSSL::X509::Name.parse('commonName=x, 2.5.4.10=y').to_a"),
        "[[\"CN\", \"x\", 12], [\"O\", \"y\", 12]]"
    );
}

#[test]
fn a_value_type_openssl_refuses_raises_name_error() {
    assert_eq!(
        inspected(
            "[0, 1, 999].map { |type| (OpenSSL::X509::Name.new.add_entry('CN', 'v', type) rescue $!.message) }"
        ),
        "[\"X509_NAME_add_entry_by_txt: nested asn1 error (Field=value, Type=X509_NAME_ENTRY)\", \"X509_NAME_add_entry_by_txt: ASN1 lib\", \"X509_NAME_add_entry_by_txt: nested asn1 error (Field=value, Type=X509_NAME_ENTRY)\"]"
    );
}

#[test]
fn a_value_not_written_as_utf8_is_kept_as_bytes() {
    assert_eq!(
        inspected(
            "held = OpenSSL::X509::Name.new\nheld.add_entry('CN', 'caf\\u00e9', 19)\n[held.to_a.first[1].encoding, held.to_a.first[2]]"
        ),
        "[#<Encoding:BINARY (ASCII-8BIT)>, 19]"
    );
}

#[test]
fn names_compare_ignoring_case_and_runs_of_spaces() {
    assert_eq!(
        inspected(
            "parsed = ->(text) { OpenSSL::X509::Name.parse(text) }\n[parsed['/CN=A'] <=> parsed['/CN=a'], parsed['/CN=a  b'] == parsed['/CN=a b'], parsed['/CN=b'] <=> parsed['/CN=a'], parsed['/CN=a'].eql?(parsed['/CN=A']), parsed['/CN=a'].hash == parsed['/CN=A'].hash, parsed['/CN=a'] <=> 'text']"
        ),
        "[0, true, 1, true, true, nil]"
    );
}

#[test]
fn a_name_is_written_in_der() {
    assert_eq!(
        inspected("OpenSSL::X509::Name.parse('/C=US/CN=a').to_der.unpack1('H*')"),
        "\"3019310b3009060355040613025553310a300806035504030c0161\""
    );
}

#[test]
fn an_entry_added_at_a_location_goes_there() {
    assert_eq!(
        inspected(
            "held = OpenSSL::X509::Name.parse('/CN=a/O=b')\nheld.add_entry('OU', 'c', loc: 1)\nheld.to_s"
        ),
        "\"/CN=a/OU=c/O=b\""
    );
}
