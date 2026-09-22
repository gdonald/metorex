// Keyed digests, derived keys, and the text written into a url or a page.

use super::*;

#[test]
fn a_keyed_digest_answers_the_value_its_definition_gives() {
    let result = run(r#"
require 'openssl'
OpenSSL::HMAC.hexdigest OpenSSL::Digest.new("SHA1"), "key",
                        "The quick brown fox jumps over the lazy dog"
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("de7c9b85b8b78aa6bc8a7a36f70a90701c9db4d9".to_string())
    );
}

#[test]
fn a_key_derived_from_a_password_is_the_same_every_time() {
    let result = run(r#"
require 'openssl'
settings = {salt: "salt", iterations: 50, length: 20, hash: "sha1"}
first = OpenSSL::KDF.pbkdf2_hmac("secret", **settings)
[first.length, first == OpenSSL::KDF.pbkdf2_hmac("secret", **settings)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[20, true]".to_string())
    );
}

#[test]
fn a_derived_key_asked_for_without_all_its_parts_is_refused() {
    let error = run_err(
        r#"
require 'openssl'
Digest.__pbkdf2__("SHA1")
"#,
    );
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_derived_key_of_an_algorithm_nothing_answers_to_is_refused() {
    let error = run_err(
        r#"
require 'openssl'
Digest.__pbkdf2__("SHA3", "pass", "salt", 2, 16)
"#,
    );
    assert!(error.contains("unknown digest algorithm"), "{error}");
}

#[test]
fn comparing_without_saying_where_two_strings_differ() {
    let result = run(r#"
require 'openssl'
[OpenSSL.fixed_length_secure_compare("abc", "abc"),
 OpenSSL.fixed_length_secure_compare("abc", "abd"),
 OpenSSL.secure_compare("held", "held")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, false, true]".to_string())
    );
}

#[test]
fn comparing_two_strings_of_different_lengths_is_refused() {
    let error = run_err(
        r#"
require 'openssl'
OpenSSL.fixed_length_secure_compare("ab", "abc")
"#,
    );
    assert!(error.contains("must be of equal length"), "{error}");
}

#[test]
fn text_written_into_a_url_carries_only_what_a_url_may_carry() {
    let result = run(r#"
require 'cgi/escape'
[CGI.escape("a b&c~"), CGI.unescape("a+b%26c"),
 CGI.escapeURIComponent("a b/c"), CGI.unescapeURIComponent("a%20b%2Fc")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[a+b%26c~, a b&c, a%20b%2Fc, a b/c]".to_string())
    );
}

#[test]
fn text_written_into_a_page_spells_out_what_the_page_reads_as_markup() {
    let result = run(r#"
require 'cgi/escape'
[CGI.escapeHTML(%[& < > " ']), CGI.unescapeHTML("&amp;&lt;&gt;&quot;&#99;&#x41;")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[&amp; &lt; &gt; &quot; &#39;, &<>\"cA]".to_string())
    );
}

#[test]
fn only_the_tags_of_the_elements_named_are_spelled_out() {
    let result = run(r#"
require 'cgi/escape'
held = CGI.escapeElement('<BR><A HREF="url"></A>', "A")
[held, CGI.unescapeElement(held, "A")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(
            "[<BR>&lt;A HREF=&quot;url&quot;&gt;&lt;/A&gt;, <BR><A HREF=\"url\"></A>]".to_string()
        )
    );
}

#[test]
fn text_written_into_a_url_has_to_be_text() {
    let error = run_err(
        r#"
require 'cgi/escape'
CGI.escape(:held)
"#,
    );
    assert!(error.contains("no implicit conversion"), "{error}");
}

#[test]
fn the_system_log_is_opened_under_a_name_and_closed_again() {
    let result = run(r#"
require 'syslog'
answered = [Syslog.opened?]
Syslog.open "metorex_test", Syslog::LOG_PID
answered << Syslog.opened? << Syslog.ident << (Syslog.options == Syslog::LOG_PID)
answered << Syslog.mask
Syslog.close
answered << Syslog.opened? << Syslog.mask
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[false, true, metorex_test, true, 255, false, nil]".to_string())
    );
}

#[test]
fn a_log_nobody_has_opened_cannot_be_closed() {
    let error = run_err(
        r#"
require 'syslog'
Syslog.close
"#,
    );
    assert!(error.contains("syslog not opened"), "{error}");
}

#[test]
fn a_log_handed_to_a_block_is_not_closed_from_inside_it() {
    let result = run(r#"
require 'syslog'
answered = nil
begin
  Syslog.open { |held| held.close }
rescue RuntimeError => problem
  answered = problem.message
end
[answered, Syslog.opened?]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[syslog opened with a block, false]".to_string())
    );
}

#[test]
fn the_mask_a_log_carries_names_the_severities_it_lets_through() {
    let result = run(r#"
require 'syslog'
Syslog.open "metorex_mask_test"
Syslog.mask = Syslog::Constants.LOG_UPTO(Syslog::LOG_WARNING)
answered = [Syslog.mask, Syslog::Constants.LOG_MASK(Syslog::LOG_DEBUG)]
Syslog.close
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[31, 128]".to_string())
    );
}

#[test]
fn a_mask_that_is_not_a_number_is_refused() {
    let error = run_err(
        r#"
require 'syslog'
Syslog.open "metorex_bad_mask"
begin
  Syslog.mask = "held"
ensure
  Syslog.close
end
"#,
    );
    assert!(error.contains("no implicit conversion"), "{error}");
}

#[test]
fn a_message_written_to_the_log_reaches_the_error_stream_when_asked() {
    let result = run(r#"
require 'syslog'
Syslog.open "metorex_write_test", Syslog::LOG_PERROR
answered = Syslog.info("held %s", "message")
Syslog.close
answered == Syslog
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn a_binding_with_no_source_behind_it_says_so() {
    let result = run(r#"
held = proc { }.binding
held.source_location
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("nil".to_string())
    );
}

#[test]
fn a_binding_asked_about_no_local_at_all_is_refused() {
    let error = run_err(r#"binding.local_variable_get"#);
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_message_written_without_all_its_parts_is_refused() {
    let error = run_err(
        r#"
require 'syslog'
Syslog.__write__("held")
"#,
    );
    assert!(error.contains("wrong number of arguments"), "{error}");
}

#[test]
fn a_keyed_digest_of_the_wider_algorithms_folds_over_a_wider_block() {
    let result = run(r#"
require 'openssl'
[OpenSSL::HMAC.hexdigest(OpenSSL::Digest.new("SHA512"), "key", "held").length,
 OpenSSL::HMAC.hexdigest(OpenSSL::Digest.new("SHA384"), "key", "held").length]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[128, 96]".to_string())
    );
}

#[test]
fn a_keyed_digest_shortens_a_key_wider_than_its_block() {
    let result = run(r#"
require 'openssl'
long = "k" * 200
held = OpenSSL::HMAC.hexdigest(OpenSSL::Digest.new("SHA1"), long, "message")
[held.length, held == OpenSSL::HMAC.hexdigest(OpenSSL::Digest.new("SHA1"), long, "message")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[40, true]".to_string())
    );
}
