//! The pattern engine on its own, matching what Ruby's patterns mean.

use metorex::regexp::{Flags, Pattern};

fn compiled(source: &str) -> Pattern {
    Pattern::compile(source, Flags::default()).expect("the pattern reads")
}

fn folded(source: &str) -> Pattern {
    Pattern::compile(
        source,
        Flags {
            folded: true,
            ..Flags::default()
        },
    )
    .expect("the pattern reads")
}

/// The text the whole match read, or None when nothing matched.
fn whole<'t>(source: &str, subject: &'t str) -> Option<&'t str> {
    compiled(source)
        .captures_at(subject, 0)
        .and_then(|found| found.get(0))
        .map(|held| held.as_str())
}

/// The text each group read, with None where a group matched nothing.
fn groups(source: &str, subject: &str) -> Vec<Option<String>> {
    let pattern = compiled(source);
    let Some(found) = pattern.captures_at(subject, 0) else {
        return Vec::new();
    };
    (0..found.len())
        .map(|index| found.get(index).map(|held| held.as_str().to_string()))
        .collect()
}

#[test]
fn matches_a_run_of_letters() {
    assert_eq!(whole("hay", "a haystack"), Some("hay"));
}

#[test]
fn reports_nothing_when_the_run_is_absent() {
    assert_eq!(whole("hay", "a stack"), None);
}

#[test]
fn a_dot_reads_any_character_but_a_newline() {
    assert_eq!(whole("a.c", "abc"), Some("abc"));
    assert_eq!(whole("a.c", "a\nc"), None);
}

#[test]
fn a_dot_reads_a_newline_under_the_multiline_flag() {
    let pattern = Pattern::compile(
        "a.c",
        Flags {
            dot_reads_newline: true,
            ..Flags::default()
        },
    )
    .expect("the pattern reads");
    assert!(pattern.is_match_at("a\nc", 0));
}

#[test]
fn a_star_takes_as_many_as_it_can() {
    assert_eq!(whole("a*", "aaab"), Some("aaa"));
}

#[test]
fn a_lazy_star_takes_as_few_as_it_can() {
    assert_eq!(whole("a*?b", "aaab"), Some("aaab"));
    assert_eq!(whole("<.+?>", "<a><b>"), Some("<a>"));
}

#[test]
fn a_possessive_star_gives_nothing_back() {
    assert_eq!(whole("a*+a", "aaa"), None);
    assert_eq!(whole("a*a", "aaa"), Some("aaa"));
}

#[test]
fn an_atomic_group_gives_nothing_back() {
    assert_eq!(whole("(?>a*)a", "aaa"), None);
}

#[test]
fn counted_repetition_reads_the_range_it_names() {
    assert_eq!(whole("a{2,3}", "aaaa"), Some("aaa"));
    assert_eq!(whole("a{2}", "aaaa"), Some("aa"));
    assert_eq!(whole("a{2,}", "aaaa"), Some("aaaa"));
    assert_eq!(whole("a{3}", "aa"), None);
}

#[test]
fn a_brace_that_counts_nothing_is_an_ordinary_character() {
    assert_eq!(whole("a{x}", "a{x}"), Some("a{x}"));
}

#[test]
fn alternation_takes_the_first_branch_that_matches() {
    assert_eq!(whole("cat|dog", "a dog"), Some("dog"));
    assert_eq!(whole("a|ab", "ab"), Some("a"));
}

#[test]
fn groups_report_what_they_read() {
    assert_eq!(
        groups("(hay(st)a)ck", "haystack"),
        vec![
            Some("haystack".to_string()),
            Some("haysta".to_string()),
            Some("st".to_string())
        ]
    );
}

#[test]
fn a_group_that_matched_nothing_reports_nothing() {
    assert_eq!(
        groups("(a)|(b)", "b"),
        vec![Some("b".to_string()), None, Some("b".to_string())]
    );
}

#[test]
fn a_named_group_carries_its_name() {
    let pattern = compiled("(?<first>a)(?<second>b)");
    let names: Vec<Option<&str>> = pattern.capture_names().collect();
    assert_eq!(names, vec![None, Some("first"), Some("second")]);
}

#[test]
fn a_name_may_be_written_on_more_than_one_group() {
    let pattern = compiled("(?<held>a)|(?<held>b)");
    let names: Vec<Option<&str>> = pattern.capture_names().collect();
    assert_eq!(names, vec![None, Some("held"), Some("held")]);
    assert!(pattern.is_match_at("b", 0));
}

#[test]
fn a_backreference_reads_what_its_group_read() {
    assert_eq!(whole(r"(ab)\1", "abab"), Some("abab"));
    assert_eq!(whole(r"(ab)\1", "abcd"), None);
}

#[test]
fn a_backreference_by_name_reads_what_that_group_read() {
    assert_eq!(whole(r"(?<held>ab)\k<held>", "abab"), Some("abab"));
}

#[test]
fn a_backreference_folds_case_when_the_pattern_does() {
    let pattern = folded(r"(ab)\1");
    assert!(pattern.is_match_at("abAB", 0));
}

#[test]
fn a_lookahead_checks_without_reading() {
    assert_eq!(whole("foo(?=bar)", "foobar"), Some("foo"));
    assert_eq!(whole("foo(?=bar)", "foobaz"), None);
}

#[test]
fn a_negative_lookahead_refuses_what_follows() {
    assert_eq!(whole("foo(?!bar)", "foobaz"), Some("foo"));
    assert_eq!(whole("foo(?!bar)", "foobar"), None);
}

#[test]
fn a_lookbehind_checks_what_came_before() {
    let pattern = compiled("(?<=foo)bar");
    let found = pattern
        .captures_at("foobar", 0)
        .and_then(|held| held.get(0))
        .expect("the pattern matches");
    assert_eq!(found.as_str(), "bar");
    assert_eq!(found.start(), 3);
    assert!(!pattern.is_match_at("bazbar", 0));
}

#[test]
fn a_negative_lookbehind_refuses_what_came_before() {
    assert!(compiled("(?<!foo)bar").is_match_at("bazbar", 0));
    assert!(!compiled("(?<!foo)bar").is_match_at("foobar", 0));
}

#[test]
fn a_lookbehind_of_more_than_one_width_still_matches() {
    assert!(compiled("(?<=ab|abc)d").is_match_at("abd", 0));
}

#[test]
fn anchors_name_the_ends_of_a_line_and_of_the_subject() {
    assert!(compiled("^b").is_match_at("a\nb", 0));
    assert!(!compiled(r"\Ab").is_match_at("a\nb", 0));
    assert!(compiled("a$").is_match_at("a\nb", 0));
    assert!(compiled(r"b\z").is_match_at("a\nb", 0));
    assert!(compiled(r"b\Z").is_match_at("a\nb\n", 0));
    assert!(!compiled(r"b\z").is_match_at("a\nb\n", 0));
}

#[test]
fn a_word_boundary_sits_between_a_word_and_what_is_not_one() {
    assert_eq!(whole(r"\bcat\b", "a cat sat"), Some("cat"));
    assert_eq!(whole(r"\bcat\b", "concatenate"), None);
    assert!(compiled(r"\Bcat\B").is_match_at("concatenate", 0));
}

#[test]
fn a_named_run_covers_the_characters_it_names() {
    assert_eq!(whole(r"\d+", "ab123cd"), Some("123"));
    assert_eq!(whole(r"\w+", " held "), Some("held"));
    assert_eq!(whole(r"\s+", "a  b"), Some("  "));
    assert_eq!(whole(r"\D+", "12ab"), Some("ab"));
}

#[test]
fn the_word_escape_covers_ascii_alone() {
    assert!(!compiled(r"^\w$").is_match_at("é", 0));
    assert!(compiled(r"^[[:alpha:]]$").is_match_at("é", 0));
}

#[test]
fn a_class_lists_and_spans_characters() {
    assert_eq!(whole("[a-c]+", "abcd"), Some("abc"));
    assert_eq!(whole("[^a-c]+", "abcd"), Some("d"));
    assert_eq!(whole("[]a]+", "]a"), Some("]a"));
    assert_eq!(whole("[a-]+", "a-"), Some("a-"));
}

#[test]
fn a_class_may_hold_what_two_classes_share() {
    assert_eq!(whole("[a-z&&[^aeiou]]+", "aeiobcd"), Some("bcd"));
}

#[test]
fn a_posix_name_stands_for_a_run() {
    assert_eq!(whole("[[:digit:]]+", "ab12"), Some("12"));
    assert_eq!(whole("[[:^digit:]]+", "ab12"), Some("ab"));
}

#[test]
fn a_hash_inside_a_class_is_not_a_comment() {
    let pattern = Pattern::compile(
        "[a#b]+",
        Flags {
            extended: true,
            ..Flags::default()
        },
    )
    .expect("the pattern reads");
    let found = pattern
        .captures_at("a#b", 0)
        .and_then(|held| held.get(0))
        .expect("the pattern matches");
    assert_eq!(found.as_str(), "a#b");
}

#[test]
fn extended_mode_drops_whitespace_and_comments() {
    let pattern = Pattern::compile(
        "a  # a comment\n  b",
        Flags {
            extended: true,
            ..Flags::default()
        },
    )
    .expect("the pattern reads");
    assert!(pattern.is_match_at("ab", 0));
}

#[test]
fn inline_flags_reach_the_rest_of_the_group() {
    assert!(compiled("(?i)abc").is_match_at("ABC", 0));
    assert!(compiled("(?i:abc)d").is_match_at("ABCd", 0));
    assert!(!compiled("(?i:abc)d").is_match_at("ABCD", 0));
    assert!(compiled("(?-i:abc)").is_match_at("abc", 0));
}

#[test]
fn folding_covers_both_cases_of_a_span() {
    assert!(folded("[a-z]+").is_match_at("ABC", 0));
}

#[test]
fn a_comment_group_says_nothing() {
    assert_eq!(whole("a(?#a note)b", "ab"), Some("ab"));
}

#[test]
fn a_group_may_be_matched_again_by_name() {
    assert!(compiled(r"(?<held>a)\g<held>").is_match_at("aa", 0));
}

#[test]
fn a_conditional_runs_the_branch_its_group_decides() {
    assert!(compiled(r"(a)?(?(1)b|c)").is_match_at("ab", 0));
    assert!(compiled(r"(a)?(?(1)b|c)").is_match_at("c", 0));
}

#[test]
fn keep_out_drops_what_came_before_it() {
    let pattern = compiled(r"foo\Kbar");
    let found = pattern
        .captures_at("foobar", 0)
        .and_then(|held| held.get(0))
        .expect("the pattern matches");
    assert_eq!(found.as_str(), "bar");
}

#[test]
fn the_search_start_names_where_the_search_began() {
    let pattern = compiled(r"\Gab");
    assert!(pattern.is_match_at("xxab", 2));
    assert!(!pattern.is_match_at("xxab", 0));
}

#[test]
fn an_empty_turn_ends_a_repetition() {
    assert_eq!(whole("(a*)*", "aaa"), Some("aaa"));
    assert_eq!(whole("(a?)*", "b"), Some(""));
}

#[test]
fn a_match_may_begin_past_the_start_of_the_subject() {
    let found = compiled("b+")
        .captures_at("aabbb", 0)
        .and_then(|held| held.get(0))
        .expect("the pattern matches");
    assert_eq!(found.start(), 2);
    assert_eq!(found.end(), 5);
}

#[test]
fn offsets_are_counted_in_bytes() {
    let found = compiled("b")
        .captures_at("éb", 0)
        .and_then(|held| held.get(0))
        .expect("the pattern matches");
    assert_eq!(found.start(), 2);
    assert_eq!(found.end(), 3);
}

#[test]
fn an_escape_names_the_character_it_spells() {
    assert!(compiled(r"\n").is_match_at("\n", 0));
    assert!(compiled(r"\x41").is_match_at("A", 0));
    assert!(compiled(r"\x{263a}").is_match_at("\u{263a}", 0));
    assert!(compiled(r"A").is_match_at("A", 0));
    assert!(compiled(r"\cJ").is_match_at("\n", 0));
}

#[test]
fn a_pattern_that_opens_a_group_it_never_closes_is_refused() {
    assert!(Pattern::compile("(hay(st)ack", Flags::default()).is_err());
    assert!(Pattern::compile("hay)stack", Flags::default()).is_err());
    assert!(Pattern::compile("(?<1a>a)", Flags::default()).is_err());
}

#[test]
fn a_pattern_that_would_run_forever_answers_instead() {
    // The shape a denial-of-service report is written around. The engine
    // stops once it has tried more pieces than any answer is waiting for.
    let pattern = compiled("^(a*)*$");
    let subject = format!("{}x", "a".repeat(40));
    assert!(!pattern.is_match_at(&subject, 0));
}
