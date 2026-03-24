use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

#[test]
fn array_length() {
    let result = run("[1, 2, 3].length");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_size_alias() {
    let result = run("[1, 2, 3].size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_push_returns_array() {
    let result = run(r#"
arr = [1, 2]
arr.push(3)
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
        ]))
    );
}

#[test]
fn array_pop_returns_last_element() {
    let result = run(r#"
arr = [1, 2, 3]
arr.pop
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_pop_mutates_array() {
    let result = run(r#"
arr = [1, 2, 3]
arr.pop
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![Object::Int(1), Object::Int(2)]))
    );
}

#[test]
fn array_pop_empty_returns_nil() {
    let result = run(r#"
arr = []
arr.pop
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_shift_returns_first_element() {
    let result = run(r#"
arr = [1, 2, 3]
arr.shift
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn array_shift_mutates_array() {
    let result = run(r#"
arr = [1, 2, 3]
arr.shift
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![Object::Int(2), Object::Int(3)]))
    );
}

#[test]
fn array_shift_empty_returns_nil() {
    let result = run(r#"
arr = []
arr.shift
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_unshift_prepends_element() {
    let result = run(r#"
arr = [2, 3]
arr.unshift(1)
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
        ]))
    );
}

#[test]
fn array_sort_integers() {
    let result = run(r#"
arr = [3, 1, 4, 1, 5, 9, 2, 6]
arr.sort
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
            Object::Int(4),
            Object::Int(5),
            Object::Int(6),
            Object::Int(9),
        ]))
    );
}

#[test]
fn array_sort_strings() {
    let result = run(r#"
arr = ["banana", "apple", "cherry"]
arr.sort
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::string("apple"),
            Object::string("banana"),
            Object::string("cherry"),
        ]))
    );
}

#[test]
fn array_sort_does_not_mutate_original() {
    let result = run(r#"
arr = [3, 1, 2]
arr.sort
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(3),
            Object::Int(1),
            Object::Int(2),
        ]))
    );
}

#[test]
fn array_reverse_integers() {
    let result = run(r#"
arr = [1, 2, 3, 4, 5]
arr.reverse
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(5),
            Object::Int(4),
            Object::Int(3),
            Object::Int(2),
            Object::Int(1),
        ]))
    );
}

#[test]
fn array_reverse_does_not_mutate_original() {
    let result = run(r#"
arr = [1, 2, 3]
arr.reverse
arr
"#);
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3),
        ]))
    );
}

#[test]
fn array_map_doubles() {
    let result = run("[1, 2, 3].map { |n| n * 2 }");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(2),
            Object::Int(4),
            Object::Int(6),
        ]))
    );
}

#[test]
fn array_select_filters_evens() {
    let result = run("[1, 2, 3, 4, 5, 6].select { |n| n % 2 == 0 }");
    assert_eq!(
        result,
        Some(Object::array(vec![
            Object::Int(2),
            Object::Int(4),
            Object::Int(6),
        ]))
    );
}

#[test]
fn array_filter_alias() {
    let result = run("[1, 2, 3, 4].filter { |n| n > 2 }");
    assert_eq!(
        result,
        Some(Object::array(vec![Object::Int(3), Object::Int(4)]))
    );
}

#[test]
fn array_reduce_with_initial() {
    let result = run("[1, 2, 3, 4, 5].reduce(0) { |acc, n| acc + n }");
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn array_reduce_without_initial() {
    let result = run("[1, 2, 3, 4, 5].reduce { |acc, n| acc + n }");
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn array_each_iterates() {
    let result = run(r#"
sum = 0
[1, 2, 3].each { |n| sum = sum + n }
sum
"#);
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn array_join_with_separator() {
    let result = run(r#"
arr = ["a", "b", "c"]
arr.join(", ")
"#);
    assert_eq!(result, Some(Object::string("a, b, c")));
}

#[test]
fn array_join_no_separator() {
    let result = run(r#"
arr = ["a", "b", "c"]
arr.join
"#);
    assert_eq!(result, Some(Object::string("abc")));
}
