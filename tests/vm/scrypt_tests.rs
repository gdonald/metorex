// OpenSSL::KDF.scrypt: the key RFC 7914 derives, and the parameters it
// refuses.

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

#[test]
fn the_empty_password_vector_of_rfc_7914_is_derived() {
    assert_eq!(
        inspected("OpenSSL::KDF.scrypt('', salt: '', N: 16, r: 1, p: 1, length: 64).unpack1('H*')"),
        "\"77d6576238657b203b19ca42c18a0497f16b4844e3074ae8dfdffa3fede21442fcd0069ded0948f8326a753a0fc81f17e8d3e0fb2e0d3628cf35e20c38d18906\""
    );
}

#[test]
fn the_salted_vector_of_rfc_7914_with_several_blocks_is_derived() {
    assert_eq!(
        inspected(
            "OpenSSL::KDF.scrypt('password', salt: 'NaCl', N: 1024, r: 8, p: 16, length: 64).unpack1('H*')"
        ),
        "\"fdbabe1c9d3472007856e7190d01e9fe7c6ad7cbc8237830e77376634b3731622eaf30d92e22a3886ff109279d9830dac727afb94a83ee6d8360cbdfa2cc0640\""
    );
}

#[test]
fn a_table_size_that_is_not_a_power_of_two_is_refused() {
    assert_eq!(
        inspected("OpenSSL::KDF.scrypt('s', salt: '', N: 3, r: 1, p: 1, length: 1)"),
        "[OpenSSL::KDF::KDFError, \"EVP_PBE_scrypt\"]"
    );
}

#[test]
fn a_negative_length_is_refused_as_a_string_size() {
    assert_eq!(
        inspected("OpenSSL::KDF.scrypt('s', salt: '', N: 2, r: 1, p: 1, length: -1)"),
        "[ArgumentError, \"negative string size (or size too big)\"]"
    );
}

#[test]
fn one_missing_keyword_is_named_alone() {
    assert_eq!(
        inspected("OpenSSL::KDF.scrypt('s', salt: '', N: 2, r: 1, p: 1)"),
        "[ArgumentError, \"missing keyword: :length\"]"
    );
}

#[test]
fn an_unknown_keyword_is_refused() {
    assert_eq!(
        inspected("OpenSSL::KDF.scrypt('s', salt: '', N: 2, r: 1, p: 1, length: 1, extra: 1)"),
        "[ArgumentError, \"unknown keyword: :extra\"]"
    );
}

#[test]
fn the_parameters_are_converted_through_to_str_and_to_int() {
    assert_eq!(
        inspected(
            "pass = Object.new\ndef pass.to_str = 'secret'\nsize = Object.new\ndef size.to_int = 2\nOpenSSL::KDF.scrypt(pass, salt: '', N: size, r: 1, p: 1, length: 4) == OpenSSL::KDF.scrypt('secret', salt: '', N: 2, r: 1, p: 1, length: 4)"
        ),
        "true"
    );
}

#[test]
fn a_password_that_is_not_a_string_is_refused() {
    assert_eq!(
        inspected("OpenSSL::KDF.scrypt(Object.new, salt: '', N: 2, r: 1, p: 1, length: 1)"),
        "[TypeError, \"no implicit conversion of Object into String\"]"
    );
}
