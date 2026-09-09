// Coverage tests for URI

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

/// Loading and running the URI library nests deeper than the stack a test
/// thread is given, so each program runs on a thread sized like the one the
/// binary itself uses.
fn on_a_deep_stack(work: impl FnOnce() -> String + Send + 'static) -> String {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(work)
        .expect("thread failed")
        .join()
        .expect("thread panicked")
}

/// The value a program answers, written the way `inspect` writes it.
fn shown(code: &str) -> String {
    let held = format!("__answered__ = begin\n{code}\nend\n__answered__.inspect");
    on_a_deep_stack(move || {
        let tokens = Lexer::new(&held).tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        match vm.execute_program(&stmts).expect("execution failed") {
            Some(Object::String(written)) => written.to_string(),
            other => panic!("expected a string, got {other:?}"),
        }
    })
}

/// The text a program answers, with the quotes `inspect` puts around a
/// string taken back off.
fn quoted(code: &str) -> String {
    let written = shown(code);
    written
        .strip_prefix('"')
        .and_then(|held| held.strip_suffix('"'))
        .expect("expected a string")
        .to_string()
}

fn run_err(code: &str) -> String {
    let held = code.to_string();
    on_a_deep_stack(move || {
        let tokens = Lexer::new(&held).tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        vm.execute_program(&stmts).unwrap_err().to_string()
    })
}

const PAGE: &str =
    "require 'uri'\npage = URI.parse(\"http://user:pass@example.com:8080/path?q=1#top\")\n";

// ── Splitting a URI ────────────────────────────────────────────────────────

#[test]
fn the_scheme_decides_which_class_carries_the_components() {
    assert_eq!(shown(&format!("{PAGE}page.class")), "URI::HTTP");
    assert_eq!(
        shown("require 'uri'\nURI.parse('https://example.com/').class"),
        "URI::HTTPS"
    );
    assert_eq!(
        shown("require 'uri'\nURI.parse('ftp://host/x').class"),
        "URI::FTP"
    );
    assert_eq!(
        shown("require 'uri'\nURI.parse('ldap://host/dn').class"),
        "URI::LDAP"
    );
    assert_eq!(
        shown("require 'uri'\nURI.parse('mailto:a@b.c').class"),
        "URI::MailTo"
    );
    assert_eq!(
        shown("require 'uri'\nURI.parse('gopher://host/x').class"),
        "URI::Generic"
    );
}

#[test]
fn every_component_reads_back_off_a_parsed_uri() {
    assert_eq!(quoted(&format!("{PAGE}page.scheme")), "http");
    assert_eq!(quoted(&format!("{PAGE}page.userinfo")), "user:pass");
    assert_eq!(quoted(&format!("{PAGE}page.user")), "user");
    assert_eq!(quoted(&format!("{PAGE}page.password")), "pass");
    assert_eq!(quoted(&format!("{PAGE}page.host")), "example.com");
    assert_eq!(shown(&format!("{PAGE}page.port")), "8080");
    assert_eq!(quoted(&format!("{PAGE}page.path")), "/path");
    assert_eq!(quoted(&format!("{PAGE}page.query")), "q=1");
    assert_eq!(quoted(&format!("{PAGE}page.fragment")), "top");
    assert_eq!(quoted(&format!("{PAGE}page.request_uri")), "/path?q=1");
    assert_eq!(
        quoted(&format!("{PAGE}page.to_s")),
        "http://user:pass@example.com:8080/path?q=1#top"
    );
}

#[test]
fn a_scheme_with_a_default_port_fills_it_in() {
    assert_eq!(shown("require 'uri'\nURI.parse('http://a/').port"), "80");
    assert_eq!(shown("require 'uri'\nURI.parse('https://a/').port"), "443");
    assert_eq!(shown("require 'uri'\nURI.parse('ftp://a/').port"), "21");
    assert_eq!(shown("require 'uri'\nURI.parse('ldap://a/').port"), "389");
    assert_eq!(shown("require 'uri'\nURI.parse('gopher://a/').port"), "nil");
}

#[test]
fn an_opaque_uri_keeps_everything_after_the_colon() {
    assert_eq!(
        quoted("require 'uri'\nURI.parse('news:comp.lang.ruby').opaque"),
        "comp.lang.ruby"
    );
    assert_eq!(shown("require 'uri'\nURI.parse('news:x').path"), "nil");
    assert_eq!(
        shown("require 'uri'\nURI.parse('news:x').hierarchical?"),
        "false"
    );
    assert_eq!(
        shown("require 'uri'\nURI.parse('a/b/c').absolute?"),
        "false"
    );
}

#[test]
fn split_answers_the_nine_components_a_uri_is_made_of() {
    assert_eq!(
        shown("require 'uri'\nURI.split('http://a:8080/p?q#f')"),
        "[\"http\", nil, \"a\", \"8080\", nil, \"/p\", nil, \"q\", \"f\"]"
    );
}

// ── The schemes with parts of their own ────────────────────────────────────

#[test]
fn an_ftp_uri_carries_a_typecode_and_a_rootless_path() {
    const FILE: &str = "require 'uri'\nheld = URI.parse('ftp://user@host/pub/ruby.tgz;type=i')\n";
    assert_eq!(quoted(&format!("{FILE}held.path")), "pub/ruby.tgz");
    assert_eq!(quoted(&format!("{FILE}held.typecode")), "i");
    assert_eq!(
        quoted(&format!("{FILE}held.to_s")),
        "ftp://user@host/pub/ruby.tgz;type=i"
    );
    assert_eq!(
        quoted("require 'uri'\nURI.parse('ftp://host/%2Ffoo').path"),
        "/foo"
    );
    assert_eq!(
        quoted("require 'uri'\nheld = URI.parse('ftp://host')\nheld.path = '/foo'\nheld.to_s"),
        "ftp://host/%2Ffoo"
    );
    assert!(
        run_err("require 'uri'\nURI.parse('ftp://host/x').typecode = 'z'")
            .contains("bad component")
    );
}

#[test]
fn an_ldap_uri_splits_its_query_into_named_fields() {
    const HELD: &str =
        "require 'uri'\nheld = URI.parse('ldap://host/o=Example,c=US?mail?sub?filter?ext')\n";
    assert_eq!(quoted(&format!("{HELD}held.dn")), "o=Example,c=US");
    assert_eq!(quoted(&format!("{HELD}held.attributes")), "mail");
    assert_eq!(quoted(&format!("{HELD}held.scope")), "sub");
    assert_eq!(quoted(&format!("{HELD}held.filter")), "filter");
    assert_eq!(quoted(&format!("{HELD}held.extensions")), "ext");
    assert!(
        run_err("require 'uri'\nURI.parse('ldap://host/dn').scope = 'nowhere'")
            .contains("bad component")
    );
}

#[test]
fn a_mailto_uri_splits_its_address_from_its_headers() {
    const HELD: &str = "require 'uri'\nheld = URI.parse('mailto:a@b.c?subject=Hi&body=There')\n";
    assert_eq!(quoted(&format!("{HELD}held.to")), "a@b.c");
    assert_eq!(
        shown(&format!("{HELD}held.headers")),
        "[[\"subject\", \"Hi\"], [\"body\", \"There\"]]"
    );
    assert_eq!(
        quoted("require 'uri'\nURI::MailTo.build(['a@b.c', ['subject=Hi']]).to_s"),
        "mailto:a@b.c?subject=Hi"
    );
    assert!(
        run_err("require 'uri'\nURI::MailTo.build(['javascript:alert()', []])")
            .contains("invalid as URI")
    );
}

// ── Joining and routing ────────────────────────────────────────────────────

#[test]
fn a_relative_reference_is_laid_over_the_base_it_is_read_against() {
    const BASE: &str = "require 'uri'\nbase = URI.parse('http://a/b/c/d;p?q')\n";
    assert_eq!(
        quoted(&format!("{BASE}(base + 'g').to_s")),
        "http://a/b/c/g"
    );
    assert_eq!(
        quoted(&format!("{BASE}(base + '../g').to_s")),
        "http://a/b/g"
    );
    assert_eq!(quoted(&format!("{BASE}(base + '/g').to_s")), "http://a/g");
    assert_eq!(quoted(&format!("{BASE}(base + '//g').to_s")), "http://g");
    assert_eq!(
        quoted(&format!("{BASE}(base + '?y').to_s")),
        "http://a/b/c/d;p?y"
    );
    assert_eq!(
        quoted(&format!("{BASE}(base + '').to_s")),
        "http://a/b/c/d;p?q"
    );
    assert_eq!(shown(&format!("{BASE}(base + 'g').class")), "URI::HTTP");
}

#[test]
fn a_relative_base_cannot_be_merged_onto() {
    assert!(run_err("require 'uri'\nURI.parse('a/b/c') + 'd'").contains("both URI are relative"));
}

#[test]
fn route_to_gives_the_shortest_way_from_one_uri_to_another() {
    const BASE: &str = "require 'uri'\nbase = URI.parse('http://a/b/c/d;p?q')\n";
    assert_eq!(
        quoted(&format!("{BASE}base.route_to('http://a/b/c/g').to_s")),
        "g"
    );
    assert_eq!(
        quoted(&format!("{BASE}base.route_to('http://a/b/g').to_s")),
        "../g"
    );
    assert_eq!(
        quoted(&format!("{BASE}base.route_to('http://a/g').to_s")),
        "../../g"
    );
    assert_eq!(
        quoted(&format!("{BASE}base.route_to('http://g').to_s")),
        "//g"
    );
    assert_eq!(quoted(&format!("{BASE}base.route_to('g:h').to_s")), "g:h");
}

#[test]
fn join_walks_a_base_through_every_reference_in_turn() {
    assert_eq!(
        quoted("require 'uri'\nURI.join('http://localhost/', 'main.rb').to_s"),
        "http://localhost/main.rb"
    );
    assert_eq!(
        quoted("require 'uri'\nURI.join('http://a/b/c/d', '../../e/f', 'g/h/../i').to_s"),
        "http://a/e/g/i"
    );
    assert_eq!(
        quoted("require 'uri'\nURI.join('http://a/b', 'http://x/y', 'z').to_s"),
        "http://x/z"
    );
    assert!(run_err("require 'uri'\nURI.join").contains("wrong number of arguments"));
}

// ── Escaping and forms ─────────────────────────────────────────────────────

#[test]
fn escaping_replaces_what_a_uri_may_not_carry() {
    assert_eq!(quoted("require 'uri'\nURI.escape('a b&c')"), "a%20b&c");
    assert_eq!(quoted("require 'uri'\nURI.unescape('a%20b%26c')"), "a b&c");
    assert_eq!(
        quoted("require 'uri'\nURI.encode_www_form_component('a b&c')"),
        "a+b%26c"
    );
    assert_eq!(
        quoted("require 'uri'\nURI.decode_www_form_component('a+b%26c')"),
        "a b&c"
    );
    assert!(
        run_err("require 'uri'\nURI.decode_www_form_component('%zz')")
            .contains("invalid %-encoding")
    );
}

#[test]
fn a_form_body_is_written_and_read_back_as_pairs() {
    assert_eq!(
        quoted("require 'uri'\nURI.encode_www_form([['name', 'ruby'], ['age', '30']])"),
        "name=ruby&age=30"
    );
    assert_eq!(
        shown("require 'uri'\nURI.decode_www_form('name=ruby&age=30')"),
        "[[\"name\", \"ruby\"], [\"age\", \"30\"]]"
    );
}

#[test]
fn extract_finds_every_uri_written_into_running_text() {
    assert_eq!(
        shown("require 'uri'\nURI.extract('see http://a/x and mailto:b@c.d today')"),
        "[\"http://a/x\", \"mailto:b@c.d\"]"
    );
    assert_eq!(
        shown("require 'uri'\nURI.extract('see http://a/x and ftp://b/y', ['ftp'])"),
        "[\"ftp://b/y\"]"
    );
    assert_eq!(
        shown("require 'uri'\nfound = []\nURI.extract('a http://b/') { |one| found.push(one) }"),
        "nil"
    );
}

// ── Comparing and writing ──────────────────────────────────────────────────

#[test]
fn two_uris_are_equal_when_their_normalized_forms_are() {
    assert_eq!(
        shown("require 'uri'\nURI('http://example.com') == URI('http://example.com/')"),
        "true"
    );
    assert_eq!(
        shown("require 'uri'\nURI('http://exAMPLE.cOm') == URI('http://example.com')"),
        "true"
    );
    assert_eq!(
        shown("require 'uri'\nURI('http://a/paTH') == URI('http://a/path')"),
        "false"
    );
    assert_eq!(
        shown("require 'uri'\nURI('http://a') == 'http://a'"),
        "false"
    );
    assert_eq!(
        quoted("require 'uri'\nURI('http://exAMPLE.cOm').normalize.to_s"),
        "http://example.com/"
    );
}

#[test]
fn the_uri_method_parses_and_passes_a_uri_through() {
    assert_eq!(shown("require 'uri'\nURI('http://a/').class"), "URI::HTTP");
    assert_eq!(
        shown("require 'uri'\nKernel::URI('http://a/').class"),
        "URI::HTTP"
    );
    assert_eq!(
        shown("require 'uri'\nheld = URI('http://a/')\nURI(held).equal?(held)"),
        "true"
    );
}

#[test]
fn a_component_that_the_scheme_has_no_room_for_is_refused() {
    assert!(run_err("require 'uri'\nURI.parse('mailto:a@b.c').host = 'x'").contains("opaque"));
    assert!(
        run_err("require 'uri'\nURI.parse('http://a').password = 'x'")
            .contains("password component depends user component")
    );
    assert!(
        run_err("require 'uri'\nURI.parse('http://a').registry = 'x'")
            .contains("can not set registry")
    );
    assert!(
        run_err("require 'uri'\nURI.parse('http://a/p').select('scheme')").contains("expected")
    );
}
