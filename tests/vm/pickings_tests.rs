// Coverage tests for the array pickings, the binary search, and the set
// grouping methods

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

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── The ways elements are picked out of an array ───────────────────────────

#[test]
fn combination_picks_every_group_of_the_given_length() {
    assert_eq!(
        run("[1, 2, 3].combination(2).to_a.inspect"),
        Some(Object::string("[[1, 2], [1, 3], [2, 3]]"))
    );
}

#[test]
fn combination_of_zero_answers_one_empty_group() {
    assert_eq!(
        run("[1, 2].combination(0).to_a.inspect"),
        Some(Object::string("[[]]"))
    );
}

#[test]
fn combination_out_of_bounds_answers_nothing() {
    assert_eq!(
        run("[1, 2].combination(5).to_a.inspect"),
        Some(Object::string("[]"))
    );
    assert_eq!(
        run("[1, 2].combination(-1).to_a.inspect"),
        Some(Object::string("[]"))
    );
}

#[test]
fn combination_reports_the_binomial_coefficient_as_its_size() {
    assert_eq!(
        run("[1, 2, 3, 4].combination(2).size"),
        Some(Object::Int(6))
    );
    assert_eq!(run("[1, 2].combination(-1).size"), Some(Object::Int(0)));
    assert_eq!(run("[].combination(0).size"), Some(Object::Int(1)));
}

#[test]
fn combination_with_a_block_answers_the_array_itself() {
    assert_eq!(
        run("a = [1, 2]\na.combination(1) { |picked| picked }.equal?(a)"),
        Some(Object::Bool(true))
    );
}

#[test]
fn permutation_picks_every_ordering() {
    assert_eq!(run("[1, 2, 3].permutation.to_a.size"), Some(Object::Int(6)));
    assert_eq!(
        run("[1, 2].permutation.to_a.inspect"),
        Some(Object::string("[[1, 2], [2, 1]]"))
    );
}

#[test]
fn permutation_truncates_a_float_length() {
    assert_eq!(
        run("[1, 2, 3].permutation(2.7).to_a.size"),
        Some(Object::Int(6))
    );
}

#[test]
fn permutation_reports_the_descending_factorial_as_its_size() {
    assert_eq!(run("[1, 2, 3].permutation(2).size"), Some(Object::Int(6)));
    assert_eq!(run("[1, 2, 3].permutation(4).size"), Some(Object::Int(0)));
    assert_eq!(run("[].permutation.size"), Some(Object::Int(1)));
}

#[test]
fn repeated_combination_may_pick_the_same_element_again() {
    assert_eq!(
        run("[1, 2].repeated_combination(2).to_a.inspect"),
        Some(Object::string("[[1, 1], [1, 2], [2, 2]]"))
    );
    assert_eq!(
        run("[].repeated_combination(2).to_a.inspect"),
        Some(Object::string("[]"))
    );
    assert_eq!(
        run("[1, 2, 3].repeated_combination(5).size"),
        Some(Object::Int(21))
    );
    assert_eq!(
        run("[1].repeated_combination(-1).size"),
        Some(Object::Int(0))
    );
}

#[test]
fn repeated_permutation_reports_a_power_as_its_size() {
    assert_eq!(
        run("[1, 2].repeated_permutation(2).to_a.inspect"),
        Some(Object::string("[[1, 1], [1, 2], [2, 1], [2, 2]]"))
    );
    assert_eq!(
        run("[1, 2, 3].repeated_permutation(4).size"),
        Some(Object::Int(81))
    );
    assert_eq!(
        run("[1].repeated_permutation(-1).size"),
        Some(Object::Int(0))
    );
    assert_eq!(
        run("[1].repeated_permutation(-1).to_a.inspect"),
        Some(Object::string("[]"))
    );
}

// ── The rows a product walks ───────────────────────────────────────────────

#[test]
fn product_walks_every_row_of_the_given_lists() {
    assert_eq!(
        run("[1, 2].product([3, 4]).inspect"),
        Some(Object::string("[[1, 3], [1, 4], [2, 3], [2, 4]]"))
    );
}

#[test]
fn product_without_arguments_wraps_each_element() {
    assert_eq!(
        run("[1, 2].product.inspect"),
        Some(Object::string("[[1], [2]]"))
    );
}

#[test]
fn product_with_an_empty_list_walks_nothing() {
    assert_eq!(
        run("[1, 2].product([]).inspect"),
        Some(Object::string("[]"))
    );
}

#[test]
fn product_with_a_block_answers_the_array_itself() {
    assert_eq!(
        run("a = [1]\na.product([2]) { |row| row }.equal?(a)"),
        Some(Object::Bool(true))
    );
}

#[test]
fn product_refuses_an_argument_that_is_not_a_list() {
    let err = run_err("[1].product(2..3)");
    assert!(err.contains("into Array"), "Error was: {}", err);
}

#[test]
fn product_refuses_a_number_of_rows_it_will_not_walk() {
    let err = run_err("a = (0..100).to_a\na.product(a, a, a, a, a)");
    assert!(err.contains("too big to product"), "Error was: {}", err);
}

// ── The search that halves a sorted array ──────────────────────────────────

#[test]
fn bsearch_answers_the_first_element_the_block_accepts() {
    assert_eq!(
        run("[0, 1, 3, 4].bsearch { |n| n >= 2 }"),
        Some(Object::Int(3))
    );
}

#[test]
fn bsearch_answers_nil_when_the_block_accepts_nothing() {
    assert_eq!(run("[0, 1, 2].bsearch { |n| n > 5 }"), Some(Object::Nil));
    assert_eq!(run("[0, 1, 2].bsearch { |n| nil }"), Some(Object::Nil));
}

#[test]
fn bsearch_reads_a_number_from_the_block_as_a_direction() {
    assert_eq!(
        run("[0, 1, 2, 3].bsearch { |n| n <=> 2 }"),
        Some(Object::Int(2))
    );
    assert_eq!(
        run("[0, 1, 3, 4].bsearch { |n| n <=> 2 }"),
        Some(Object::Nil)
    );
}

#[test]
fn bsearch_refuses_a_block_answering_something_it_cannot_read() {
    let err = run_err(r#"[1].bsearch { "1" }"#);
    assert!(err.contains("wrong argument type"), "Error was: {}", err);
}

#[test]
fn bsearch_without_a_block_answers_an_enumerator_of_unknown_size() {
    assert_eq!(
        run("[1].bsearch.class.name"),
        Some(Object::string("Enumerator"))
    );
    assert_eq!(run("[1].bsearch.size"), Some(Object::Nil));
    assert_eq!(run("[1].bsearch_index.size"), Some(Object::Nil));
}

#[test]
fn bsearch_index_answers_the_position_rather_than_the_element() {
    assert_eq!(
        run("[0, 1, 3, 4].bsearch_index { |n| n >= 2 }"),
        Some(Object::Int(2))
    );
    assert_eq!(
        run("[0, 1, 3].bsearch_index { |n| 3 <=> n }"),
        Some(Object::Int(2))
    );
    assert_eq!(
        run("[0, 1, 3].bsearch_index { |n| false }"),
        Some(Object::Nil)
    );
}

#[test]
fn bsearch_index_refuses_a_block_answering_something_it_cannot_read() {
    let err = run_err("[1].bsearch_index { Object.new }");
    assert!(err.contains("wrong argument type"), "Error was: {}", err);
}

// ── The subsets a set is cut into ──────────────────────────────────────────

#[test]
fn set_classify_groups_elements_under_the_block_answer() {
    assert_eq!(
        run(r#"Set["ab", "cd", "e"].classify { |word| word.length }.inspect"#),
        Some(Object::string(
            "{2 => Set[\"ab\", \"cd\"], 1 => Set[\"e\"]}"
        ))
    );
}

#[test]
fn set_classify_without_a_block_answers_an_enumerator() {
    assert_eq!(
        run("Set[1, 2].classify.class.name"),
        Some(Object::string("Enumerator"))
    );
}

#[test]
fn set_divide_answers_a_set_of_subsets() {
    assert_eq!(
        run("Set[1, 2, 3].divide { |n| n.odd? }.map { |group| group.to_a.sort }.sort.inspect"),
        Some(Object::string("[[1, 3], [2]]"))
    );
}

#[test]
fn set_divide_with_two_parameters_groups_what_the_block_relates() {
    assert_eq!(
        run("Set[1, 2, 5].divide { |a, b| (a - b).abs == 1 }.map { |g| g.to_a.sort }.sort.inspect"),
        Some(Object::string("[[1, 2], [5]]"))
    );
}

#[test]
fn set_divide_without_a_block_answers_an_enumerator() {
    assert_eq!(
        run("Set[1, 2].divide.class.name"),
        Some(Object::string("Enumerator"))
    );
}

#[test]
fn set_flatten_opens_up_the_sets_it_holds() {
    assert_eq!(
        run("Set[1, Set[2, Set[3]]].flatten.to_a.sort.inspect"),
        Some(Object::string("[1, 2, 3]"))
    );
}

#[test]
fn set_flatten_refuses_a_set_that_reaches_itself() {
    let err = run_err("s = Set.new\ns << s\ns.flatten");
    assert!(err.contains("recursive Set"), "Error was: {}", err);
}

#[test]
fn set_flatten_in_place_answers_nil_when_nothing_changed() {
    assert_eq!(run("Set[1, 2].flatten!"), Some(Object::Nil));
    assert_eq!(
        run("s = Set[1, Set[2]]\ns.flatten!.to_a.sort.inspect"),
        Some(Object::string("[1, 2]"))
    );
}

// ── A hash read as a lambda ────────────────────────────────────────────────

#[test]
fn hash_to_proc_reads_one_key_per_call() {
    assert_eq!(run("{a: 1}.to_proc.call(:a)"), Some(Object::Int(1)));
    assert_eq!(run("{a: 1}.to_proc.call(:b)"), Some(Object::Nil));
    assert_eq!(run("{a: 1}.to_proc.lambda?"), Some(Object::Bool(true)));
    assert_eq!(run("{a: 1}.to_proc.arity"), Some(Object::Int(1)));
}

#[test]
fn hash_to_proc_passes_as_a_block() {
    assert_eq!(
        run("{a: 1, b: 2}.to_proc.then { |reader| [:a, :b].map(&reader) }.inspect"),
        Some(Object::string("[1, 2]"))
    );
}

// ── The runs Enumerable cuts, and the walk that carries a second value ─────

#[test]
fn slice_after_refuses_a_pattern_beside_a_block() {
    let err = run_err("[1, 2].slice_after(1) { |n| n }");
    assert!(err.contains("both pattern and block"), "Error was: {}", err);
}

#[test]
fn slice_after_refuses_neither_a_pattern_nor_a_block() {
    let err = run_err("[1, 2].slice_after");
    assert!(
        err.contains("wrong number of arguments"),
        "Error was: {}",
        err
    );
}

#[test]
fn slice_before_refuses_a_pattern_beside_a_block() {
    let err = run_err("[1, 2].slice_before(1) { |n| n }");
    assert!(err.contains("both pattern and block"), "Error was: {}", err);
}

#[test]
fn enumerator_with_object_hands_the_block_a_second_value() {
    assert_eq!(
        run("[1, 2].each.with_object([]) { |n, memo| memo.push(n * 2) }.inspect"),
        Some(Object::string("[2, 4]"))
    );
}

#[test]
fn enumerator_with_object_without_a_block_answers_an_enumerator() {
    assert_eq!(
        run("[1, 2].each.with_object(\"x\").to_a.inspect"),
        Some(Object::string("[[1, \"x\"], [2, \"x\"]]"))
    );
}

// ── A lazy walk pulled one element at a time ───────────────────────────────

#[test]
fn lazy_next_pulls_only_what_it_needs() {
    assert_eq!(
        run(
            "walked = 0\nwalk = [1, 2, 3].lazy.select { |n| walked += 1; true }\nwalk.next\nwalked"
        ),
        Some(Object::Int(1))
    );
}

#[test]
fn lazy_peek_does_not_advance_the_walk() {
    assert_eq!(
        run("walk = [1, 2].lazy.map { |n| n }\nwalk.peek\nwalk.peek"),
        Some(Object::Int(1))
    );
}

#[test]
fn lazy_rewind_starts_the_walk_over() {
    assert_eq!(
        run("walk = [1, 2].lazy.map { |n| n }\nwalk.next\nwalk.rewind.next"),
        Some(Object::Int(1))
    );
}

#[test]
fn lazy_next_past_the_end_reports_the_walk_is_over() {
    let err = run_err("walk = [1].lazy.map { |n| n }\nwalk.next\nwalk.next");
    assert!(
        err.contains("iteration reached an end"),
        "Error was: {}",
        err
    );
}

#[test]
fn lazy_chunk_without_a_block_takes_the_one_given_to_each() {
    assert_eq!(
        run("[1, 1, 2].lazy.chunk.each { |n| n }.force.inspect"),
        Some(Object::string("[[1, [1, 1]], [2, [2]]]"))
    );
}

#[test]
fn lazy_flat_map_opens_up_another_lazy_walk() {
    assert_eq!(
        run("[1, 2].lazy.flat_map { |n| [n, n].lazy }.force.inspect"),
        Some(Object::string("[1, 1, 2, 2]"))
    );
}
