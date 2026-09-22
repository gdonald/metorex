// The directives `pack` and `unpack` refuse.

use super::*;

#[test]
fn lgamma_grows_without_bound_at_infinity() {
    let result = run("Math.lgamma(Float::INFINITY)");
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[Infinity, 1]".to_string())
    );
}

#[test]
fn lgamma_approaches_the_pole_at_zero_from_either_side() {
    let result = run("[Math.lgamma(0.0)[1], Math.lgamma(-0.0)[1]]");
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, -1]".to_string())
    );
}

#[test]
fn waitall_takes_no_arguments() {
    let error = run_err("Process.waitall(0)");
    assert!(
        error.contains("wrong number of arguments"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn mkfifo_takes_the_mode_it_is_given() {
    let result = run(r#"
path = "/tmp/metorex_fifo_mode"
File.delete(path) if File.exist?(path)
File.mkfifo(path, 0644)
answer = File.pipe?(path)
File.delete(path)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn mkfifo_refuses_a_to_path_that_is_not_a_name() {
    let error = run_err(
        r#"
class NotAName
  def to_path
    42
  end
end
File.mkfifo(NotAName.new)
"#,
    );
    assert!(error.contains("String"), "unexpected error: {}", error);
}

#[test]
fn an_unknown_pack_directive_is_named_in_the_refusal() {
    let error = run_err(r#"[1].pack("K")"#);
    assert!(
        error.contains("unknown pack directive 'K'"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn an_unknown_unpack_directive_is_named_in_the_refusal() {
    let error = run_err(r#""abc".unpack("K")"#);
    assert!(
        error.contains("unknown unpack directive 'K'"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_width_modifier_is_refused_where_the_directive_has_no_platform_width() {
    let error = run_err(r#""abcdefgh".unpack("a!")"#);
    assert!(
        error.contains("'!' allowed only after types sSiIlLqQjJ"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn packing_fewer_items_than_the_format_asks_for_is_refused() {
    let error = run_err(r#"[].pack("N")"#);
    assert!(
        error.contains("too few arguments"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn skipping_past_the_end_of_the_string_is_refused() {
    let error = run_err(r#""ab".unpack("x4C")"#);
    assert!(
        error.contains("outside of string"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn stepping_back_further_than_the_string_reaches_is_refused() {
    let error = run_err(r#""abcd".unpack("CX*C")"#);
    assert!(
        error.contains("outside of string"),
        "unexpected error: {}",
        error
    );
}
