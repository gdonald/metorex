// Symbols, array algebra, and a body that ends in an `if`.

use super::*;

#[test]
fn a_symbol_reports_the_symbol_class() {
    let result = run(":name.class.name");
    assert_eq!(result, Some(Object::string("Symbol".to_string())));
}

#[test]
fn a_symbol_is_not_a_string() {
    let result = run("String === :name");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn a_symbol_keeps_the_character_level_methods() {
    let result = run(":alpha.length");
    assert_eq!(result, Some(Object::Int(5)));
}

// ── Array intersection and union ─────────────────────────────────────────────

#[test]
fn array_intersection_keeps_left_order_without_duplicates() {
    let result = run("([1, 2, 3, 2] & [2, 3, 4]).inspect");
    assert_eq!(result, Some(Object::string("[2, 3]".to_string())));
}

#[test]
fn array_union_keeps_first_seen_order_without_duplicates() {
    let result = run("([1, 2, 3, 2] | [2, 3, 4]).inspect");
    assert_eq!(result, Some(Object::string("[1, 2, 3, 4]".to_string())));
}

// ── A body ending in `if` under a method-level rescue ────────────────────────

#[test]
fn a_method_with_a_rescue_clause_returns_its_trailing_if() {
    let result = run(r#"
def choose(flag)
  if flag
    "yes"
  else
    "no"
  end
rescue => error
  "rescued"
end
choose(true)
"#);
    assert_eq!(result, Some(Object::string("yes".to_string())));
}

#[test]
fn a_begin_block_returns_its_trailing_unless() {
    let result = run(r#"
begin
  unless false
    "taken"
  end
rescue => error
  "rescued"
end
"#);
    assert_eq!(result, Some(Object::string("taken".to_string())));
}
