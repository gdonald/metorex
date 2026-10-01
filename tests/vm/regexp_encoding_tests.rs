// The encoding a pattern matches in, the strings it refuses, and the pieces
// a match cuts out of a string that pairs its bytes.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements).expect("execution failed")
}

fn text(value: &str) -> Option<Object> {
    Some(Object::string(value))
}

fn ints(values: &[i64]) -> Option<Object> {
    Some(Object::array(
        values.iter().map(|value| Object::Int(*value)).collect(),
    ))
}

/// The message of the error `code` raises, rescued in the program.
fn refusal(code: &str) -> Option<Object> {
    run(&format!(
        "begin\n  {code}\n  :matched\nrescue Encoding::CompatibilityError, ArgumentError => error\n  error.message\nend"
    ))
}

#[test]
fn an_euc_jp_string_held_as_bytes_is_matched_one_character_at_a_time() {
    assert_eq!(
        run("held = \"\\xC3\\xA9\".b.force_encoding('EUC-JP')\n/./e.match(held)[0].bytes"),
        ints(&[0xC3, 0xA9])
    );
}

#[test]
fn an_euc_jp_string_held_as_text_is_matched_through_its_bytes() {
    assert_eq!(
        run(
            "held = \"a\\u00e9b\".dup.force_encoding('EUC-JP')\nfound = /./.match(held, 1)\n[found.begin(0), found[0].bytes]"
        ),
        Some(Object::array(vec![
            Object::Int(1),
            Object::array(vec![Object::Int(0xC3), Object::Int(0xA9)])
        ]))
    );
}

#[test]
fn a_shift_jis_string_is_matched_one_character_at_a_time() {
    assert_eq!(
        run(
            "held = \"a\\x82\\xA0\\xB1\".b.force_encoding('Windows-31J')\n[(/..\\z/s =~ held), $~[0].bytes]"
        ),
        Some(Object::array(vec![
            Object::Int(1),
            Object::array(vec![
                Object::Int(0x82),
                Object::Int(0xA0),
                Object::Int(0xB1)
            ])
        ]))
    );
}

#[test]
fn a_pair_no_table_names_still_counts_as_one_character() {
    assert_eq!(
        run("held = \"\\xFE\\xFEz\".b.force_encoding('EUC-JP')\nheld =~ /z/"),
        Some(Object::Int(1))
    );
}

#[test]
fn a_string_matched_with_a_symbol_on_the_other_side_matches_the_name() {
    assert_eq!(run("/ab/ =~ :cab"), Some(Object::Int(1)));
}

#[test]
fn string_match_on_an_euc_jp_string_keeps_the_encoding_of_its_pieces() {
    assert_eq!(
        run("held = \"\\xC3\\xA9\".b.force_encoding('EUC-JP')\nheld.match(/./)[0].encoding.name"),
        text("EUC-JP")
    );
}

#[test]
fn indexing_an_euc_jp_string_counts_characters() {
    assert_eq!(
        run(
            "held = \"a\\xC3\\xA9b\".b.force_encoding('EUC-JP')\n[held[1].bytes, held[1..2].bytes, held[1, 1].bytes, held[-1]]"
        ),
        Some(Object::array(vec![
            Object::array(vec![Object::Int(0xC3), Object::Int(0xA9)]),
            Object::array(vec![
                Object::Int(0xC3),
                Object::Int(0xA9),
                Object::Int(0x62)
            ]),
            Object::array(vec![Object::Int(0xC3), Object::Int(0xA9)]),
            Object::string("b")
        ]))
    );
}

#[test]
fn a_piece_of_a_string_keeps_its_encoding() {
    assert_eq!(
        run("'abc'.dup.force_encoding('ISO-8859-1')[0, 2].encoding.name"),
        text("ISO-8859-1")
    );
}

#[test]
fn a_shift_jis_string_inspects_a_pair_by_its_bytes_and_a_lone_byte_by_itself() {
    assert_eq!(
        run("\"a\\x82\\xA0\\xB1\".b.force_encoding('Windows-31J').inspect"),
        text("\"a\\x{82A0}\\xB1\"")
    );
}

#[test]
fn the_last_encoding_letter_written_decides() {
    assert_eq!(
        run("[/foo/ensuensuens == /foo/s, /foo/un.options, /#{'a'}/ens.encoding.name]"),
        Some(Object::array(vec![
            Object::Bool(true),
            Object::Int(32),
            Object::string("Windows-31J")
        ]))
    );
}

#[test]
fn an_interpolated_literal_matches_in_the_encoding_its_letter_names() {
    assert_eq!(
        run("[/#{'a'}/e.encoding.name, /#{'a'}/u.encoding.name, /#{'a'}/.encoding.name]"),
        Some(Object::array(vec![
            Object::string("EUC-JP"),
            Object::string("UTF-8"),
            Object::string("US-ASCII")
        ]))
    );
}

#[test]
fn a_regexp_allocated_without_a_pattern_is_binary() {
    assert_eq!(run("Regexp.allocate.encoding.name"), text("ASCII-8BIT"));
}

#[test]
fn a_string_its_encoding_cannot_read_is_refused() {
    assert_eq!(
        refusal("\"\\x80\".dup.force_encoding('UTF-8') =~ /./"),
        text("invalid byte sequence in UTF-8")
    );
}

#[test]
fn a_string_in_an_encoding_that_spells_ascii_another_way_is_refused() {
    assert_eq!(
        refusal("/a/.match(' '.encode('UTF-16LE'))"),
        text("incompatible encoding regexp match (US-ASCII regexp with UTF-16LE string)")
    );
}

#[test]
fn a_pattern_fixed_to_an_encoding_refuses_text_past_ascii_in_another() {
    assert_eq!(
        refusal(
            "Regexp.new(''.dup.force_encoding('US-ASCII'), Regexp::FIXEDENCODING).match?(\"\\u00e9\")"
        ),
        text("incompatible encoding regexp match (US-ASCII regexp with UTF-8 string)")
    );
}

#[test]
fn a_pattern_fixed_to_an_encoding_that_spells_ascii_another_way_is_refused() {
    assert_eq!(
        refusal(
            "\"a\".match(Regexp.new(''.dup.force_encoding('UTF-16LE'), Regexp::FIXEDENCODING))"
        ),
        text("incompatible encoding regexp match (UTF-16LE regexp with UTF-8 string)")
    );
}

#[test]
fn a_fixed_pattern_matches_ascii_text_in_another_encoding() {
    assert_eq!(
        run("/a/u =~ 'cat'.dup.force_encoding('ISO-8859-1')"),
        Some(Object::Int(1))
    );
}

#[test]
fn a_pattern_matches_a_string_in_its_own_encoding() {
    assert_eq!(
        run("/./e =~ 'x'.dup.force_encoding('EUC-JP')"),
        Some(Object::Int(0))
    );
}

#[test]
fn a_string_subclass_is_checked_the_way_a_string_is() {
    assert_eq!(
        refusal("Class.new(String).new(\"\\x80\").force_encoding('UTF-8').match?(/./)"),
        text("invalid byte sequence in UTF-8")
    );
}

/// Run `code` with `$stderr` gathered, and answer what was written there.
fn warnings_from(code: &str) -> String {
    let wrapped = format!(
        "gathered = []\ncollector = Object.new\ncollector.define_singleton_method(:write) {{ |text| gathered << text }}\n$stderr = collector\n{code}\n$stderr = STDERR\ngathered.join"
    );
    match run(&wrapped) {
        Some(Object::String(written)) => written.as_str().to_string(),
        other => panic!("expected the warnings, got {other:?}"),
    }
}

#[test]
fn a_binary_pattern_matched_against_text_past_ascii_is_warned_about() {
    assert!(
        warnings_from("/./n.match(\"\\u00e9\")")
            .contains("warning: historical binary regexp match /.../n against UTF-8 string")
    );
}

#[test]
fn a_binary_pattern_matched_against_bytes_or_ascii_is_not_warned_about() {
    assert_eq!(
        warnings_from("/./n.match(\"\\xFF\".b)\n/./n.match('abc')"),
        ""
    );
}

#[test]
fn escaped_bytes_in_a_literal_keep_a_valid_run_as_its_character() {
    assert_eq!(
        run("\"\\303\\251 \\xFF\".bytes"),
        ints(&[0xC3, 0xA9, 0x20, 0xFF])
    );
}

#[test]
fn escaped_bytes_in_an_interpolated_literal_stand_for_themselves() {
    assert_eq!(
        run(
            "held = \"\\u00e9\"\nmade = \"#{held} \\xFF\"\n[made.bytes, made.encoding.name, made.valid_encoding?]"
        ),
        Some(Object::array(vec![
            Object::array(vec![
                Object::Int(0xC3),
                Object::Int(0xA9),
                Object::Int(0x20),
                Object::Int(0xFF)
            ]),
            Object::string("UTF-8"),
            Object::Bool(false)
        ]))
    );
}

#[test]
fn a_meta_escape_names_a_byte() {
    assert_eq!(run("\"\\M-a\".bytes"), ints(&[0xE1]));
}

#[test]
fn a_heredoc_reads_octal_hexadecimal_and_unicode_escapes() {
    assert_eq!(
        run("<<~TEXT\n  \\101\\x42\\u0043\\u{44 45}\\0\nTEXT"),
        text("ABCDE\0\n")
    );
}

#[test]
fn a_heredoc_with_escaped_bytes_holds_those_bytes() {
    assert_eq!(
        run("held = <<~TEXT\n  \\303\\251 \\xFF\nTEXT\n[held.bytes, held.valid_encoding?]"),
        Some(Object::array(vec![
            ints(&[0xC3, 0xA9, 0x20, 0xFF, 0x0A]).expect("bytes"),
            Object::Bool(false)
        ]))
    );
}

#[test]
fn a_heredoc_with_interpolation_keeps_its_escaped_bytes() {
    assert_eq!(
        run("held = <<~TEXT\n  #{1} \\xFF\nTEXT\nheld.bytes"),
        ints(&[0x31, 0x20, 0xFF, 0x0A])
    );
}

#[test]
fn a_heredoc_in_a_binary_source_holds_bytes_in_that_encoding() {
    assert_eq!(
        run(
            "# encoding: binary\nheld = <<~TEXT\n  \\303\\251\nTEXT\n[held.encoding.name, held.bytes]"
        ),
        Some(Object::array(vec![
            Object::string("ASCII-8BIT"),
            ints(&[0xC3, 0xA9, 0x0A]).expect("bytes")
        ]))
    );
}

#[test]
fn a_heredoc_octal_escape_past_255_keeps_its_low_byte() {
    assert_eq!(run("<<~TEXT.bytes\n  \\777\nTEXT"), ints(&[0xFF, 0x0A]));
}
