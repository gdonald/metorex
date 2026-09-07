// Coverage tests for Data value objects and the method-shape reporting

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

const MEASURE: &str = "Measure = Data.define(:amount, :unit)\n";

// ── Building a Data class ──────────────────────────────────────────────────

#[test]
fn data_define_answers_a_class_naming_its_members() {
    assert_eq!(
        run("Data.define(:a, :b).members.inspect"),
        Some(Object::string("[:a, :b]"))
    );
    assert_eq!(
        run("Data.define(\"a\", :b).members.inspect"),
        Some(Object::string("[:a, :b]"))
    );
    assert_eq!(
        run("Data.define.members.inspect"),
        Some(Object::string("[]"))
    );
}

#[test]
fn data_define_takes_a_block_of_methods() {
    assert_eq!(
        run("K = Data.define(:a) do\n  def twice\n    a * 2\n  end\nend\nK.new(3).twice"),
        Some(Object::Int(6))
    );
}

#[test]
fn data_define_refuses_a_member_that_is_not_a_name() {
    let err = run_err("Data.define(1)");
    assert!(
        err.contains("not a symbol nor a string"),
        "Error was: {}",
        err
    );
}

#[test]
fn data_define_refuses_a_repeated_member() {
    let err = run_err("Data.define(:a, :a)");
    assert!(err.contains("duplicate member"), "Error was: {}", err);
}

#[test]
fn data_subclass_of_data_itself_names_no_members() {
    assert_eq!(
        run("Class.new(Data).respond_to?(:members)"),
        Some(Object::Bool(false))
    );
}

// ── Building one value ─────────────────────────────────────────────────────

#[test]
fn data_reads_positional_and_keyword_arguments_the_same_way() {
    assert_eq!(
        run(&format!("{MEASURE}Measure.new(42, \"km\").amount")),
        Some(Object::Int(42))
    );
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(amount: 42, unit: \"km\").amount"
        )),
        Some(Object::Int(42))
    );
    assert_eq!(
        run(&format!("{MEASURE}Measure[42, \"km\"].amount")),
        Some(Object::Int(42))
    );
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure[amount: 42, unit: \"km\"].amount"
        )),
        Some(Object::Int(42))
    );
}

#[test]
fn data_value_cannot_be_changed_once_made() {
    assert_eq!(
        run(&format!("{MEASURE}Measure.new(1, \"m\").frozen?")),
        Some(Object::Bool(true))
    );
}

#[test]
fn data_reports_every_member_a_call_left_out() {
    let err = run_err(&format!("{MEASURE}Measure.new"));
    assert!(
        err.contains("missing keywords: :amount, :unit"),
        "Error was: {}",
        err
    );
    let err = run_err(&format!("{MEASURE}Measure.new(unit: \"km\")"));
    assert!(
        err.contains("missing keyword: :amount"),
        "Error was: {}",
        err
    );
}

#[test]
fn data_refuses_a_name_it_has_no_member_for() {
    let err = run_err(&format!(
        "{MEASURE}Measure.new(amount: 1, unit: \"m\", pace: 2)"
    ));
    assert!(err.contains("unknown keyword: :pace"), "Error was: {}", err);
}

#[test]
fn data_refuses_more_arguments_than_it_has_members() {
    let err = run_err(&format!("{MEASURE}Measure.new(1, 2, 3)"));
    assert!(
        err.contains("wrong number of arguments"),
        "Error was: {}",
        err
    );
}

#[test]
fn data_refuses_positional_and_keyword_arguments_together() {
    let err = run_err(&format!("{MEASURE}Measure.new(1, unit: \"m\")"));
    assert!(
        err.contains("wrong number of arguments"),
        "Error was: {}",
        err
    );
}

// ── Reading one value ──────────────────────────────────────────────────────

#[test]
fn data_answers_its_members_and_values() {
    assert_eq!(
        run(&format!("{MEASURE}Measure.new(1, \"m\").members.inspect")),
        Some(Object::string("[:amount, :unit]"))
    );
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").deconstruct.inspect"
        )),
        Some(Object::string("[1, \"m\"]"))
    );
    assert_eq!(
        run(&format!("{MEASURE}Measure.new(1, \"m\").to_h.inspect")),
        Some(Object::string("{amount: 1, unit: \"m\"}"))
    );
}

#[test]
fn data_to_h_takes_a_block_that_rewrites_each_pair() {
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").to_h {{ |name, value| [value, name] }}.inspect"
        )),
        Some(Object::string("{1 => :amount, \"m\" => :unit}"))
    );
}

#[test]
fn data_to_h_refuses_a_block_answering_the_wrong_shape() {
    let err = run_err(&format!(
        "{MEASURE}Measure.new(1, \"m\").to_h {{ |name| [name] }}"
    ));
    assert!(
        err.contains("element has wrong array length"),
        "Error was: {}",
        err
    );
    let err = run_err(&format!(
        "{MEASURE}Measure.new(1, \"m\").to_h {{ |name| 5 }}"
    ));
    assert!(err.contains("wrong element type"), "Error was: {}", err);
}

#[test]
fn data_deconstruct_keys_answers_only_what_was_asked_for() {
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").deconstruct_keys([:amount]).inspect"
        )),
        Some(Object::string("{amount: 1}"))
    );
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").deconstruct_keys(nil).inspect"
        )),
        Some(Object::string("{amount: 1, unit: \"m\"}"))
    );
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").deconstruct_keys([0, -1]).inspect"
        )),
        Some(Object::string("{0 => 1, -1 => \"m\"}"))
    );
}

#[test]
fn data_deconstruct_keys_stops_at_the_first_name_it_has_no_member_for() {
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").deconstruct_keys([:pace, :amount]).inspect"
        )),
        Some(Object::string("{}"))
    );
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").deconstruct_keys([:amount, :amount, :unit]).inspect"
        )),
        Some(Object::string("{}"))
    );
}

#[test]
fn data_deconstruct_keys_refuses_anything_but_a_list_or_nil() {
    let err = run_err(&format!(
        "{MEASURE}Measure.new(1, \"m\").deconstruct_keys(:amount)"
    ));
    assert!(err.contains("expected Array or nil"), "Error was: {}", err);
    let err = run_err(&format!(
        "{MEASURE}Measure.new(1, \"m\").deconstruct_keys([[]])"
    ));
    assert!(err.contains("into Integer"), "Error was: {}", err);
}

#[test]
fn data_renders_itself_with_its_class_and_values() {
    assert_eq!(
        run(&format!("{MEASURE}Measure.new(1, \"m\").inspect")),
        Some(Object::string("#<data Measure amount=1, unit=\"m\">"))
    );
    assert_eq!(
        run("Data.define(:a).new(1).inspect"),
        Some(Object::string("#<data a=1>"))
    );
    assert_eq!(
        run("Data.define.new.inspect"),
        Some(Object::string("#<data>"))
    );
}

#[test]
fn data_renders_a_value_that_reaches_itself_without_running_forever() {
    assert_eq!(
        run(&format!(
            "{MEASURE}held = Measure.allocate\nheld.send(:initialize, amount: 1, unit: held)\nheld.inspect"
        )),
        Some(Object::string(
            "#<data Measure amount=1, unit=#<data Measure:...>>"
        ))
    );
}

// ── Comparing values ───────────────────────────────────────────────────────

#[test]
fn data_values_match_when_their_class_and_members_match() {
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\") == Measure.new(1, \"m\")"
        )),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").eql?(Measure.new(1, \"m\"))"
        )),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\") == Measure.new(1, \"km\")"
        )),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run(&format!("{MEASURE}Measure.new(1, \"m\") == 5")),
        Some(Object::Bool(false))
    );
}

#[test]
fn data_values_of_different_classes_never_match() {
    assert_eq!(
        run("Data.define(:a).new(1) == Data.define(:a).new(1)"),
        Some(Object::Bool(false))
    );
    assert_eq!(
        run("Data.define(:a).new(1).hash == Data.define(:a).new(1).hash"),
        Some(Object::Bool(false))
    );
}

#[test]
fn data_values_that_match_report_the_same_hash() {
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").hash == Measure.new(1, \"m\").hash"
        )),
        Some(Object::Bool(true))
    );
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").hash == Measure.new(2, \"m\").hash"
        )),
        Some(Object::Bool(false))
    );
}

#[test]
fn data_compares_a_value_that_reaches_itself() {
    assert_eq!(
        run(&format!(
            "{MEASURE}a = Measure.allocate\na.send(:initialize, amount: 1, unit: a)\nb = Measure.allocate\nb.send(:initialize, amount: 1, unit: b)\na == b"
        )),
        Some(Object::Bool(true))
    );
}

// ── Copying with changes ───────────────────────────────────────────────────

#[test]
fn data_with_answers_a_fresh_value_carrying_the_changes() {
    assert_eq!(
        run(&format!(
            "{MEASURE}Measure.new(1, \"m\").with(unit: \"km\").to_h.inspect"
        )),
        Some(Object::string("{amount: 1, unit: \"km\"}"))
    );
    assert_eq!(
        run(&format!(
            "{MEASURE}held = Measure.new(1, \"m\")\nheld.with.equal?(held)"
        )),
        Some(Object::Bool(true))
    );
}

#[test]
fn data_with_refuses_positional_arguments() {
    let err = run_err(&format!("{MEASURE}Measure.new(1, \"m\").with(2, \"km\")"));
    assert!(
        err.contains("wrong number of arguments (given 2, expected 0)"),
        "Error was: {}",
        err
    );
}

// ── What a class defining its own new does ─────────────────────────────────

#[test]
fn a_class_defining_new_builds_its_instances_that_way() {
    assert_eq!(
        run("class Made\n  def self.new(count)\n    count * 2\n  end\nend\nMade.new(4)"),
        Some(Object::Int(8))
    );
}

// ── A constant bound inside a module carries that namespace ────────────────

#[test]
fn a_constant_bound_inside_a_conditional_still_names_its_namespace() {
    assert_eq!(
        run("module Outer\n  if true\n    Inner = Class.new\n  end\nend\nOuter::Inner.inspect"),
        Some(Object::string("Outer::Inner"))
    );
}

// ── super with keyword arguments ───────────────────────────────────────────

#[test]
fn super_passes_keyword_arguments_through() {
    assert_eq!(
        run(
            "class Base\n  def take(**kw)\n    kw\n  end\nend\nclass Sub < Base\n  def take(width:, height:)\n    super(width: width, area: width * height)\n  end\nend\nSub.new.take(width: 2, height: 3).inspect"
        ),
        Some(Object::string("{width: 2, area: 6}"))
    );
}
