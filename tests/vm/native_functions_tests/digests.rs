// The digest algorithms, and how each reads its message.

use super::*;

#[test]
fn each_digest_answers_the_value_its_algorithm_is_defined_to_give() {
    let result = run(r#"
require 'digest'
[
  Digest::MD5.hexdigest(""),
  Digest::MD5.hexdigest("abc"),
  Digest::SHA1.hexdigest(""),
  Digest::SHA256.hexdigest("abc"),
  Digest::SHA384.hexdigest(""),
  Digest::SHA512.hexdigest("")
].join(" ")
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(
            [
                "d41d8cd98f00b204e9800998ecf8427e",
                "900150983cd24fb0d6963f7d28e17f72",
                "da39a3ee5e6b4b0d3255bfef95601890afd80709",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
                concat!(
                    "38b060a751ac96384cd9327eb1b1e36a21fdb71114be0743",
                    "4c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b"
                ),
                concat!(
                    "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce",
                    "47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
                ),
            ]
            .join(" ")
        )
    );
}

#[test]
fn a_digest_taken_of_a_message_leaves_the_object_blank() {
    let result = run(r#"
require 'digest'
running = Digest::SHA256.new
running << "test"
taken = running.hexdigest("abc")
[taken == Digest::SHA256.hexdigest("abc"), running.hexdigest == Digest::SHA256.hexdigest("")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true]".to_string())
    );
}

#[test]
fn a_digest_that_names_no_algorithm_refuses_to_answer() {
    let error = run_err(
        r#"
require 'digest'
Digest::Class.new.finish
"#,
    );
    assert!(error.contains("does not name an algorithm"), "{error}");
}

#[test]
fn the_base_digest_protocol_refuses_the_work_it_does_not_do() {
    let result = run(r#"
require 'digest'
holder = ::Class.new do
  include Digest::Instance
end
refused = []
begin
  holder.new.update "test"
rescue RuntimeError
  refused << :update
end
[:finish, :reset, :block_length].each do |name|
  begin
    holder.new.send name
  rescue RuntimeError
    refused << name
  end
end
refused
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:update, :finish, :reset, :block_length]".to_string())
    );
}

#[test]
fn a_digest_reads_a_file_as_the_bytes_it_holds() {
    let result = run(r#"
require 'digest'
held = "/tmp/metorex_digest_file_test.txt"
File.write(held, "abc")
answer = Digest::SHA256.file(held).hexdigest
File.delete(held)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_string())
    );
}

#[test]
fn a_digest_asks_an_object_that_is_not_a_string_how_to_read_one() {
    let result = run(r#"
require 'digest'
named = Object.new
def named.to_str
  "abc"
end
Digest::SHA256.hexdigest(named)
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_string())
    );
}

#[test]
fn a_digest_refuses_a_message_it_cannot_read_a_string_out_of() {
    let error = run_err(
        r#"
require 'digest'
Digest.hexencode(nil)
"#,
    );
    assert!(error.contains("no implicit conversion"), "{error}");
}

#[test]
fn the_sha2_family_is_named_by_its_bit_length() {
    let result = run(r#"
require 'digest'
[256, 384, 512].map { |bits| Digest::SHA2.hexdigest("abc", bits).length }
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[64, 96, 128]".to_string())
    );
}

#[test]
fn the_sha2_family_refuses_a_bit_length_it_has_no_algorithm_for() {
    let error = run_err(
        r#"
require 'digest'
Digest::SHA2.new(224)
"#,
    );
    assert!(error.contains("unsupported bit length"), "{error}");
}

#[test]
fn reading_a_file_as_bytes_keeps_what_a_text_read_would_lose() {
    let result = run(r#"
held = "/tmp/metorex_binread_test.bin"
File.binwrite(held, ["00ff10"].pack("H*"))
answer = File.binread(held).each_char.map { |byte| byte.ord }
File.delete(held)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[0, 255, 16]".to_string())
    );
}

#[test]
fn reading_a_directory_as_bytes_is_refused() {
    let error = run_err(r#"File.binread("/tmp")"#);
    assert!(error.contains("Is a directory"), "{error}");
}

#[test]
fn reading_a_name_no_file_answers_to_as_bytes_is_refused() {
    let error = run_err(r#"File.binread("/tmp/metorex_no_such_file_at_all.bin")"#);
    assert!(error.contains("No such file or directory"), "{error}");
}

#[test]
fn base64_packed_with_a_zero_count_carries_no_line_breaks() {
    let result = run(r#"[["abc"].pack("m"), ["abc"].pack("m0")]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[YWJj\n, YWJj]".to_string())
    );
}

#[test]
fn a_string_says_it_already_is_one() {
    let result = run(r#""abc".respond_to?(:to_str)"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn writing_bytes_to_a_file_answers_how_many_it_wrote() {
    let result = run(r#"
held = "/tmp/metorex_binwrite_count_test.bin"
written = File.binwrite(held, ["00ff10"].pack("H*"))
File.delete(held)
written
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("3".to_string()));
}

#[test]
fn writing_bytes_to_a_name_no_directory_holds_is_refused() {
    let error = run_err(r#"File.binwrite("/tmp/metorex_no_such_dir/held.bin", "x")"#);
    assert!(error.contains("No such file or directory"), "{error}");
}

#[test]
fn a_digest_of_an_algorithm_metorex_has_no_implementation_of_is_refused() {
    let error = run_err("require 'digest'\nDigest.__digest__(\"SHA3\", \"abc\")");
    assert!(error.contains("unknown digest algorithm SHA3"), "{error}");
}

#[test]
fn a_digest_asked_for_without_an_algorithm_and_a_message_is_refused() {
    let error = run_err("require 'digest'\nDigest.__digest__(\"MD5\")");
    assert!(error.contains("wrong number of arguments"), "{error}");
}
