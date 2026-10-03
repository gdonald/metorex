// Defining classes, and what an instance reports about itself.

use super::super::run_example;
use super::*;
#[test]
fn test_oop_top_level_include_execution() {
    let expected = "true\nhi from greeter\n";
    let output = run_example("oop/top_level_include.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_simple_execution() {
    let expected = "AB\n";
    let output = run_example("oop/test/super_simple.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_str_execution() {
    let expected = "Person: Alice\n";
    let output = run_example("oop/test/str.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_repr_execution() {
    let expected = "Point(0, 0)\n";
    let output = run_example("oop/test/repr.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_special_methods_execution() {
    let expected = "Book: Ruby Guide\nMagazine: Tech Monthly\nnext_value\n";
    let output = run_example("oop/special_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_iter_execution() {
    let expected = "next\n";
    let output = run_example("oop/test/iter.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_method_missing_execution() {
    let expected = "bar\n42\n1\n2\n3\n";
    let output = run_example("oop/test/method_missing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_scope_resolution_execution() {
    let expected = "1\n100\n";
    let output = run_example("oop/scope_resolution/scope_resolution.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_scope_resolution_parens_execution() {
    let expected = "1\n100\n";
    let output = run_example("oop/scope_resolution/scope_resolution_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_class_self_new_execution() {
    let expected = "make: self=Foo\ninit called, @x=42\nmake: inst.class=Foo\nf.class=Foo\n";
    let output = run_example("oop/class/self_new.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_class_reopen_execution() {
    let output = run_example("oop/class/reopen.rb");
    assert_eq!(output, "bar\nbaz\n");
}

#[test]
fn test_oop_toplevel_class_reopen_execution() {
    let expected = ":reopened_toplevel\n:toplevel_module\nfalse\nfalse\nWrapper::Parent\n";
    let output = run_example("oop/toplevel_class_reopen.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_keyword_method_names_execution() {
    let expected = ":original\n:original\n:original\n[:alias, :meth]\n";
    let output = run_example("oop/keyword_method_names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_keyword_method_names_parens_execution() {
    let expected = ":original\n:original\n:original\n[:alias, :meth]\n";
    let output = run_example("oop/keyword_method_names_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_set_temporary_name_execution() {
    let expected = "nil\ntrue\n\"fake_name\"\nfake_name\n\"Template[\x27foo.rb\x27]\"\nnil\ntrue\n\"host::Inner\"\nnil\n\"\": empty class/module name\n\"Object\": the temporary name must not be a constant path to avoid confusion\n\"A::B\": the temporary name must not be a constant path to avoid confusion\n\"::A\": the temporary name must not be a constant path to avoid confusion\ncan\'t change permanent name\n";
    let output = run_example("oop/set_temporary_name.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_set_temporary_name_parens_execution() {
    let expected = "nil\ntrue\n\"fake_name\"\nfake_name\n\"Template[\x27foo.rb\x27]\"\nnil\ntrue\n\"host::Inner\"\nnil\n\"\": empty class/module name\n\"Object\": the temporary name must not be a constant path to avoid confusion\n\"A::B\": the temporary name must not be a constant path to avoid confusion\n\"::A\": the temporary name must not be a constant path to avoid confusion\ncan\'t change permanent name\n";
    let output = run_example("oop/set_temporary_name_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_struct_basics_execution() {
    let expected = "1\n2\n[1, 2]\n[:x, :y]\n2\n1\n2\n1\n10\n20\n#<struct Point x=10, y=20>\ntrue\nfalse\nnil\n[:x, :y]\n[10, 20]\n10\n20\nx=10\ny=20\n42\ntrue\nexample.com\n80\nstruct size differs\nno member 'missing' in struct\n1\n10\n[10, 20]\na\n[\"a\", \"b\"]\n#<struct Pair left=\"a\", right=\"b\">\n";
    let output = run_example("oop/struct_basics.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_nested_constant_lookup_execution() {
    let expected = "true\nfalse\n3\nCatalog::Storage::Shelf\n";
    let output = run_example("oop/nested_constant_lookup.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_reopened_core_operator_execution() {
    let expected = concat!("3\n", "3\n", "[2]\n", "\"hello\"\n");
    let output = run_example("oop/reopened_core_operator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_reopened_core_operator_parens_execution() {
    let expected = concat!("3\n", "3\n", "[2]\n", "\"hello\"\n");
    let output = run_example("oop/reopened_core_operator_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_inspect_instance_variables_execution() {
    let expected = concat!(
        "#<Connection:0x @host=\"localhost\", @port=5432, @open=true>\n",
        "#<Connection:0x>\n",
        "#<Bare:0x>\n",
        "#<Chosen:0x @shown=\"yes\">\n",
        "#<NoneChosen:0x>\n",
        "Expected #instance_variables_to_inspect to return an Array or nil, but it returned Hash\n",
        "[#<Connection:0x @host=\"localhost\", @port=5432, @open=true>]\n",
        "{at: #<Connection:0x @host=\"localhost\", @port=5432, @open=true>}\n",
        "#<Connection:0x @host=\"localhost\", @port=5432, @open=true>\n",
    );
    let output = run_example("oop/inspect_instance_variables.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_inspect_instance_variables_parens_execution() {
    let expected = concat!(
        "#<Connection:0x @host=\"localhost\", @port=5432, @open=true>\n",
        "#<Connection:0x>\n",
        "#<Bare:0x>\n",
        "#<Chosen:0x @shown=\"yes\">\n",
        "#<NoneChosen:0x>\n",
        "Expected #instance_variables_to_inspect to return an Array or nil, but it returned Hash\n",
        "[#<Connection:0x @host=\"localhost\", @port=5432, @open=true>]\n",
        "{at: #<Connection:0x @host=\"localhost\", @port=5432, @open=true>}\n",
        "#<Connection:0x @host=\"localhost\", @port=5432, @open=true>\n",
    );
    let output = run_example("oop/inspect_instance_variables_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_class_variable_home_execution() {
    let output = run_example("oop/class_variable_home.rb");
    assert_eq!(output, CLASS_VARIABLE_HOME_OUTPUT);
}

#[test]
fn test_oop_class_variable_home_no_parens_execution() {
    let output = run_example("oop/class_variable_home_no_parens.rb");
    assert_eq!(output, CLASS_VARIABLE_HOME_OUTPUT);
}

#[test]
fn test_oop_class_body_clauses_execution() {
    let output = run_example("oop/class_body_clauses.rb");
    assert_eq!(output, CLASS_BODY_CLAUSES_OUTPUT);
}

#[test]
fn test_oop_class_body_clauses_no_parens_execution() {
    let output = run_example("oop/class_body_clauses_no_parens.rb");
    assert_eq!(output, CLASS_BODY_CLAUSES_OUTPUT);
}

#[test]
fn test_oop_definition_values_execution() {
    let output = run_example("oop/definition_values.rb");
    assert_eq!(output, DEFINITION_VALUES_OUTPUT);
}

#[test]
fn test_oop_definition_values_no_parens_execution() {
    let output = run_example("oop/definition_values_no_parens.rb");
    assert_eq!(output, DEFINITION_VALUES_OUTPUT);
}

/// The expected output of both `oop/class_definition_rules` variants.
const CLASS_DEFINITION_RULES_OUTPUT: &str = concat!(
    "PORT_NUMBER is not a class\n",
    "Settings is not a class\n",
    "can't make subclass of singleton class\n",
    "Object: superclass mismatch for class AuditRecord\n",
    "BasicObject: superclass mismatch for class AuditRecord\n",
    "Record\n",
    "[:@count]\n",
    "[]\n",
    ":body_value\n",
    ":singleton_value\n",
    ":module_value\n",
    "\"ScopedLog\"\n",
    ":info\n",
    "false\n",
    ":from_singleton_body\n",
);

#[test]
fn test_oop_class_definition_rules_execution() {
    let output = run_example("oop/class_definition_rules.rb");
    assert_eq!(output, CLASS_DEFINITION_RULES_OUTPUT);
}

#[test]
fn test_oop_class_definition_rules_no_parens_execution() {
    let output = run_example("oop/class_definition_rules_no_parens.rb");
    assert_eq!(output, CLASS_DEFINITION_RULES_OUTPUT);
}

/// The expected output of both `oop/nested_class_names` variants.
const NESTED_CLASS_NAMES_OUTPUT: &str = concat!(
    "Parser\n",
    "Parser\n",
    "\"parser\"\n",
    "true\n",
    "false\n",
    "false\n",
    "false\n",
    "[4, 3, 2, 1]\n",
);

#[test]
fn test_oop_nested_class_names_execution() {
    let output = run_example("oop/nested_class_names.rb");
    assert_eq!(output, NESTED_CLASS_NAMES_OUTPUT);
}

#[test]
fn test_oop_nested_class_names_no_parens_execution() {
    let output = run_example("oop/nested_class_names_no_parens.rb");
    assert_eq!(output, NESTED_CLASS_NAMES_OUTPUT);
}
