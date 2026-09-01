use super::run_example;

#[test]
fn test_metaprogramming_top_level_define_method_execution() {
    let expected = "true\nboing\nfalse\nfalse\n";
    let output = run_example("metaprogramming/top_level_define_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_anonymous_class_execution() {
    let expected = "foo\nfoo\nbaz\nModule\n";
    let output = run_example("metaprogramming/anonymous_class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_implicit_blocks_execution() {
    let expected = r#"Howdy, Alice!
Hey, Bob!
Hello, Charlie!
Iteration: 0
Iteration: 1
Iteration: 2
no block
got a block
10
20
1
4
9
[1, 4, 9, 16]
[2, 4, 6]
1
2
3
4
5
"#;
    let output = run_example("metaprogramming/implicit_blocks.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_blocks_as_objects_execution() {
    let expected = r#"=== Blocks as First-Class Objects ===

1. Assigning blocks to variables:
double.call(5) = 10

2. Multiple parameter blocks:
add.call(3, 7) = 10

3. Passing blocks as arguments to functions:
apply_twice(increment, 5) = 7

4. Returning blocks from functions (closures):
times_three.call(4) = 12
times_ten.call(4) = 40

5. Blocks capturing variables from outer scope:
First call: 1
Second call: 2
Third call: 3

6. Partial application pattern:
Hello, Alice!
Goodbye, Bob!

=== Blocks are truly first-class objects! ===
"#;

    let output = run_example("metaprogramming/blocks_as_objects.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_eval_execution() {
    let expected = "true\na widget\n42\n2\ntrue\nA WIDGET\n[\"custom.rb\", 102]\n:ok\n";
    let output = run_example("metaprogramming/class_eval.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_eval_parens_execution() {
    let expected = "true\na widget\n42\n2\ntrue\nA WIDGET\n[\"custom.rb\", 102]\n:ok\n";
    let output = run_example("metaprogramming/class_eval_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_exec_execution() {
    let expected = "gadget\n42\n2\ntag\n42\n";
    let output = run_example("metaprogramming/class_exec.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_exec_parens_execution() {
    let expected = "gadget\n42\n2\ntag\n42\n";
    let output = run_example("metaprogramming/class_exec_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_get_source_execution() {
    let expected = "speak\nfetch\ntrue\nspeak\npurr\npurr\n";
    let output = run_example("metaprogramming/get_source.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_method_execution() {
    let expected = "Hello, World!\n10\n12\nHi there!\nzero\none\ntwo\ntrue\n";
    let output = run_example("metaprogramming/define_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_ast_inspection_execution() {
    let expected = "add\n2\n1\nBinaryOp\n+\n1\nBinaryOp\n*\n1\n1\n";
    let output = run_example("metaprogramming/ast/inspection.rb");
    assert_eq!(output, expected);
}

// 10.2.1 — Method Delegation

#[test]
fn test_metaprogramming_advanced_method_delegation() {
    let expected = r#"=== Simple Delegation ===
[LOG] Processing: input data
processed: input data

=== Dynamic Delegation ===
[LOG] delegated message

=== Forwarding with Hooks ===
Before: process
[LOG] Processing: test data
After: process -> processed: test data
"#;
    let output = run_example("metaprogramming/advanced/method/delegation.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_advanced_method_delegation_no_parens() {
    let expected = r#"=== Simple Delegation ===
[LOG] Processing: input data
processed: input data

=== Dynamic Delegation ===
[LOG] delegated message

=== Forwarding with Hooks ===
Before: process
[LOG] Processing: test data
After: process -> processed: test data
"#;
    let output = run_example("metaprogramming/advanced/method/delegation_no_parens.rb");
    assert_eq!(output, expected);
}

// 10.2.2 — Aspect-Oriented Programming

#[test]
fn test_metaprogramming_advanced_aspect_oriented() {
    let expected = r#"=== Before/After Aspects ===
>> entering calculation
<< exiting calculation => 5
>> entering string op
<< exiting string op => HELLO

=== Timing Aspect ===
Starting data processing...
Finished data processing

=== Retry Aspect ===
Attempt 1...
Attempt 2...
Success on attempt 2
"#;
    let output = run_example("metaprogramming/advanced/aspect_oriented.rb");
    assert_eq!(output, expected);
}

// 10.2.3 — Lazy Evaluation

#[test]
fn test_metaprogramming_advanced_lazy_evaluation() {
    let expected = r#"=== Lazy Evaluation ===
Created lazy value
Computed yet? false
  (computing expensive value...)
First access: 1764
Computed now? true
Second access: 1764

=== Lazy Chaining ===
Pipeline created (nothing computed yet)
  (step 1: loading data...)
  (step 2: transforming data...)
  (step 3: formatting result...)
Final result: Total: 60
Accessing again (cached): Total: 60
"#;
    let output = run_example("metaprogramming/advanced/lazy_evaluation.rb");
    assert_eq!(output, expected);
}

// 10.2.4 — Memoization

#[test]
fn test_metaprogramming_advanced_memoization() {
    let expected = r#"=== Memoized Fibonacci ===
fib(0) = 0
fib(1) = 1
fib(2) = 1
fib(3) = 2
fib(4) = 3
fib(5) = 5
fib(6) = 8
fib(7) = 13
fib(8) = 21
fib(9) = 34
fib(10) = 55
Cache entries: 11

=== Generic Memoizer ===
First calls:
  (computing 5 * 5)
25
  (computing 3 * 3)
9
25
Second calls (cached):
25
9
Cache size: 2
"#;
    let output = run_example("metaprogramming/advanced/memoization.rb");
    assert_eq!(output, expected);
}

// 10.2.5 — Custom Iterators

#[test]
fn test_metaprogramming_advanced_custom_iterators() {
    let expected = r#"=== each_with_index ===
0: apple
1: banana
2: cherry

=== times ===
Iteration 0
Iteration 1
Iteration 2
Iteration 3
Iteration 4

=== reduce ===
Sum: 15
Product: 120

=== flat_map ===
[1, 2, 3, 4, 5, 6]

=== take_while ===
[2, 4, 6]
"#;
    let output = run_example("metaprogramming/advanced/custom_iterators.rb");
    assert_eq!(output, expected);
}

// 10.2.6 — Method Chaining DSL

#[test]
fn test_metaprogramming_advanced_method_chaining_dsl() {
    let output = run_example("metaprogramming/advanced/method/chaining_dsl.rb");
    assert!(
        output.contains(
            "SELECT * FROM users WHERE age > 18 AND active = true ORDER BY name LIMIT 10"
        )
    );
    assert!(output.contains("SELECT * FROM products WHERE price < 100"));
    assert!(output.contains("<h1>Welcome</h1>"));
    assert!(output.contains("<p>This is a paragraph.</p>"));
    assert!(output.contains("host = localhost"));
    assert!(output.contains("port = 8080"));
}

// 14.2 — Method Missing

#[test]
fn test_metaprogramming_method_missing_execution() {
    let expected = r#"=== Dynamic Attribute Access ===
Alice
30
engineer
unknown: email

=== Ghost Methods ===
Called hello with 0 arg(s)
Called add with 2 arg(s)
Called greet with 3 arg(s)

=== Flexible Calculator ===
6
30
unknown operation: multiply

=== Selective ===
I am real
ghost: fake_method

=== Inherited method_missing ===
Base caught: anything
Base caught: whatever
"#;
    let output = run_example("metaprogramming/method_missing/method_missing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_method_missing_no_parens_execution() {
    let expected = r#"=== Dynamic Attribute Access ===
Alice
30
engineer
unknown: email

=== Ghost Methods ===
Called hello with 0 arg(s)
Called add with 2 arg(s)
Called greet with 3 arg(s)

=== Flexible Calculator ===
6
30
unknown operation: multiply

=== Selective ===
I am real
ghost: fake_method

=== Inherited method_missing ===
Base caught: anything
Base caught: whatever
"#;
    let output = run_example("metaprogramming/method_missing/method_missing_no_parens.rb");
    assert_eq!(output, expected);
}

// 14.3 — Runtime Class Modification

#[test]
fn test_metaprogramming_class_modification_execution() {
    let expected = r#"=== alias_method ===
Hello, Alice!
Hello, Bob!

=== remove_method ===
moving
no method: speak

=== undef_method ===
Base farewell
undefined: greet

=== multiple aliases ===
HELLO
HELLO
HELLO

=== module_function ===
14
12

=== remove_method with inheritance ===
Parent greet
"#;
    let output = run_example("metaprogramming/class_modification/class_modification.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_modification_no_parens_execution() {
    let expected = r#"=== alias_method ===
Hello, Alice!
Hello, Bob!

=== remove_method ===
moving
no method: speak

=== undef_method ===
Base farewell
undefined: greet

=== multiple aliases ===
HELLO
HELLO
HELLO

=== module_function ===
14
12

=== remove_method with inheritance ===
Parent greet
"#;
    let output = run_example("metaprogramming/class_modification/class_modification_no_parens.rb");
    assert_eq!(output, expected);
}

// 14.4 — Reflection and Introspection

#[test]
fn test_metaprogramming_reflection_execution() {
    let expected = r#"=== class ===
Dog
String
Integer
Array

=== instance_of? ===
true
false

=== is_a? ===
true
true

=== respond_to? ===
true
true
false

=== methods ===
2

=== send ===
Woof!
Fetching ball

=== send with symbol ===
Woof!
Fetching stick

=== send on built-in ===
HELLO
5
"#;
    let output = run_example("metaprogramming/reflection/reflection.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_reflection_no_parens_execution() {
    let expected = r#"=== class ===
Dog
String
Integer
Array

=== instance_of? ===
true
false

=== is_a? ===
true
true

=== respond_to? ===
true
true
false

=== methods ===
2

=== send ===
Woof!
Fetching ball

=== send with symbol ===
Woof!
Fetching stick

=== send on built-in ===
HELLO
5
"#;
    let output = run_example("metaprogramming/reflection/reflection_no_parens.rb");
    assert_eq!(output, expected);
}

// 14.5 — AST Manipulation

#[test]
fn test_metaprogramming_ast_manipulation_execution() {
    let expected = r#"=== eval ===
6
50
30

=== eval define method ===
14

=== eval define class ===
(3, 4)

=== parse ===
Array
1

=== code generation ===
7
7
30

=== runtime modification ===
8
15
"#;
    let output = run_example("metaprogramming/ast/manipulation.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_execution() {
    let expected = "10\n20\n105\n";
    let output = run_example("metaprogramming/instance_exec/instance_exec.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_parens_execution() {
    let expected = "10\n20\n105\n";
    let output = run_example("metaprogramming/instance_exec/instance_exec_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_ast_manipulation_no_parens_execution() {
    let expected = r#"=== eval ===
6
50
30

=== eval define method ===
14

=== eval define class ===
(3, 4)

=== parse ===
Array
1

=== code generation ===
7
7
30

=== runtime modification ===
8
15
"#;
    let output = run_example("metaprogramming/ast/manipulation_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_refinement_execution() {
    let output = run_example("metaprogramming/refinement.rb");
    assert_eq!(output, "HELLO!\n");
}

#[test]
fn test_metaprogramming_class_variable_defined_execution() {
    let expected = "true\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\n";
    let output = run_example("metaprogramming/class_variable_defined.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_defined_parens_execution() {
    let expected = "true\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\n";
    let output = run_example("metaprogramming/class_variable_defined_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_get_execution() {
    let expected = "7\n7\nyes\n7\nhere\nmissing raises NameError\n";
    let output = run_example("metaprogramming/class_variable_get.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_get_parens_execution() {
    let expected = "7\n7\nyes\n7\nhere\nmissing raises NameError\n";
    let output = run_example("metaprogramming/class_variable_get_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_set_execution() {
    let expected =
        "on\non\n3\n3\nfrozen Class raises FrozenError\nfrozen Module raises FrozenError\n";
    let output = run_example("metaprogramming/class_variable_set.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_set_parens_execution() {
    let expected =
        "on\non\n3\n3\nfrozen Class raises FrozenError\nfrozen Module raises FrozenError\n";
    let output = run_example("metaprogramming/class_variable_set_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variables_execution() {
    let expected =
        "[:@@base, :@@shared]\n[:@@derived, :@@base, :@@shared]\n[:@@derived]\n[:@@flag]\n";
    let output = run_example("metaprogramming/class_variables.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variables_parens_execution() {
    let expected =
        "[:@@base, :@@shared]\n[:@@derived, :@@base, :@@shared]\n[:@@derived]\n[:@@flag]\n";
    let output = run_example("metaprogramming/class_variables_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_added_hook_execution() {
    let expected = "[:TEST]\n[:TEST, :SECOND]\n[:TEST, :SECOND, :Autoload]\n[:TEST, :SECOND, :Autoload, :Child]\n[:TEST, :SECOND, :Autoload, :Child, :DIRECT]\n";
    let output = run_example("metaprogramming/const_added_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_added_hook_parens_execution() {
    let expected = "[:TEST]\n[:TEST, :SECOND]\n[:TEST, :SECOND, :Autoload]\n[:TEST, :SECOND, :Autoload, :Child]\n[:TEST, :SECOND, :Autoload, :Child, :DIRECT]\n";
    let output = run_example("metaprogramming/const_added_hook_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_caller_locations_lineno_execution() {
    let expected = "true\ntrue\n";
    let output = run_example("metaprogramming/caller_locations_lineno.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_defined_execution() {
    let expected = "true\ntrue\ntrue\nfalse\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\nfalse\nNameError\nNameError\n";
    let output = run_example("metaprogramming/const_defined.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_defined_parens_execution() {
    let expected = "true\ntrue\ntrue\nfalse\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\nfalse\nNameError\nNameError\n";
    let output = run_example("metaprogramming/const_defined_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_get_execution() {
    let expected = ":from_parent\n:from_module\n:top\n:from_parent\n:top\n[:missing, :ANYTHING]\n:FROM_PARENT\nNameError\n";
    let output = run_example("metaprogramming/const_get.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_get_parens_execution() {
    let expected = ":from_parent\n:from_module\n:top\n:from_parent\n:top\n[:missing, :ANYTHING]\n:FROM_PARENT\nNameError\n";
    let output = run_example("metaprogramming/const_get_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_missing_execution() {
    let expected = "handled Anything\nhandled Direct\nuninitialized constant Bare::Nope\n:Nope\nuninitialized constant Bare::AlsoMissing\n";
    let output = run_example("metaprogramming/const_missing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_missing_parens_execution() {
    let expected = "handled Anything\nhandled Direct\nuninitialized constant Bare::Nope\n:Nope\nuninitialized constant Bare::AlsoMissing\n";
    let output = run_example("metaprogramming/const_missing_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_set_execution() {
    let expected =
        "nil\nNamedRoot\nNamedRoot::B\nNamedRoot::B::C\ntrue\n41\nNameError\nFrozenError\n";
    let output = run_example("metaprogramming/const_set.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_set_parens_execution() {
    let expected =
        "nil\nNamedRoot\nNamedRoot::B\nNamedRoot::B::C\ntrue\n41\nNameError\nFrozenError\n";
    let output = run_example("metaprogramming/const_set_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_source_location_execution() {
    let expected = "true\ntrue\ntrue\n[\"virtual.rb\", 100]\n[]\nnil\ntrue\nnil\n";
    let output = run_example("metaprogramming/const_source_location.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_const_source_location_parens_execution() {
    let expected = "true\ntrue\ntrue\n[\"virtual.rb\", 100]\n[]\nnil\ntrue\nnil\n";
    let output = run_example("metaprogramming/const_source_location_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_module_constants_execution() {
    let expected = "[:DEEP_CONST, :OWN_CONST, :SHALLOW_CONST]\n[:OWN_CONST]\n[:OWN_CONST]\n[:DEEP_CONST, :SHALLOW_CONST]\nW\ntrue\ntrue\ntrue\n";
    let output = run_example("metaprogramming/module_constants.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_module_constants_parens_execution() {
    let expected = "[:DEEP_CONST, :OWN_CONST, :SHALLOW_CONST]\n[:OWN_CONST]\n[:OWN_CONST]\n[:DEEP_CONST, :SHALLOW_CONST]\nW\ntrue\ntrue\ntrue\n";
    let output = run_example("metaprogramming/module_constants_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_block_param_defaults_execution() {
    let expected = "[5, 1]\n[5, 6]\n[1, 1]\n[1, 2]\nArgumentError\nArgumentError\n9\n";
    let output = run_example("metaprogramming/block_param_defaults.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_block_param_defaults_parens_execution() {
    let expected = "[5, 1]\n[5, 6]\n[1, 1]\n[1, 2]\nArgumentError\nArgumentError\n9\n";
    let output = run_example("metaprogramming/block_param_defaults_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_method_from_callable_execution() {
    let expected = concat!(
        "[1, 2]\n",
        ":bar\n",
        ":module_method\n",
        "3\n",
        ":named\n",
        "[:public_one]\n",
        "[:initialize, :private_one]\n",
        "wrong argument type String (expected Proc/Method/UnboundMethod)\n",
        "FrozenError\n"
    );
    let output = run_example("metaprogramming/define_method_from_callable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_method_from_callable_no_parens_execution() {
    let expected = "[1, 2]\n:bar\n:module_method\n3\n:named\n";
    let output = run_example("metaprogramming/define_method_from_callable_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_singleton_method_execution() {
    let expected = "42\n20\nbuilt\nhi\nfalse\n";
    let output = run_example("metaprogramming/define_singleton_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_singleton_method_no_parens_execution() {
    let expected = "42\nhey\n";
    let output = run_example("metaprogramming/define_singleton_method_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_deprecate_constant_execution() {
    let expected = concat!(
        "true\n",
        "false\n",
        ":old\n",
        ":kept\n",
        ":old\n",
        "true\n",
        ":old\n",
        "NameError\n",
        "private\n",
        ":hidden\n"
    );
    let output = run_example("metaprogramming/deprecate_constant.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_deprecate_constant_no_parens_execution() {
    let expected = "2\n1\nprivate\n";
    let output = run_example("metaprogramming/deprecate_constant_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_method_lambda_control_flow_execution() {
    let expected = "42\n42\n42\n[:first, :second]\n1\n[1, 2]\n";
    let output = run_example("metaprogramming/define_method_lambda_control_flow.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_method_lambda_control_flow_no_parens_execution() {
    let expected = "42\n42\n[:first, :second]\n";
    let output = run_example("metaprogramming/define_method_lambda_control_flow_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_singleton_method_sources_execution() {
    let expected =
        "[:LIMIT]\ntrue\nfrom parent\nnot defined on the parent\nhello\ntrue\nFrozenError\n";
    let output = run_example("metaprogramming/define_singleton_method_sources.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_eval_source_execution() {
    let expected = concat!(
        "42\n42\nHOLA\n",
        "wrong number of arguments (given 2, expected 0)\n",
        "wrong number of arguments (given 0, expected 1..3)\n",
        "wrong number of arguments (given 4, expected 1..3)\n"
    );
    let output = run_example("metaprogramming/instance_eval_source/strings.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_eval_source_parens_execution() {
    let expected = concat!(
        "42\n42\nHOLA\n",
        "wrong number of arguments (given 2, expected 0)\n",
        "wrong number of arguments (given 0, expected 1..3)\n",
        "wrong number of arguments (given 4, expected 1..3)\n"
    );
    let output = run_example("metaprogramming/instance_eval_source/strings_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_receivers_execution() {
    let expected = concat!(
        "7\n10\n3\n",
        "no block given (yield)\n",
        "can't define singleton\n",
        "can't define singleton\n",
        "-1\n-1\n"
    );
    let output = run_example("metaprogramming/instance_exec/receivers.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_receivers_parens_execution() {
    let expected = concat!(
        "7\n10\n3\n",
        "no block given (yield)\n",
        "can't define singleton\n",
        "can't define singleton\n",
        "-1\n-1\n"
    );
    let output = run_example("metaprogramming/instance_exec/receivers_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_method_missing_visibility_execution() {
    let expected = concat!(
        "handled hidden with []\n",
        "handled shielded with [1, 2]\n",
        "handled absent with [:arg]\n",
        "private method 'hidden' called for an instance of Plain\n",
        ":hidden\n",
        "true\n",
        "undefined method 'absent' for an instance of Passthrough\n",
        ":absent\n"
    );
    let output = run_example("metaprogramming/method_missing/visibility.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_method_missing_visibility_parens_execution() {
    let expected = concat!(
        "handled hidden with []\n",
        "handled shielded with [1, 2]\n",
        "handled absent with [:arg]\n",
        "private method 'hidden' called for an instance of Plain\n",
        ":hidden\n",
        "true\n",
        "undefined method 'absent' for an instance of Passthrough\n",
        ":absent\n"
    );
    let output = run_example("metaprogramming/method_missing/visibility_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_added_execution() {
    let expected = concat!(
        "object gained singleton_method_added\n",
        "object gained by_def\n",
        "object gained in_singleton_body\n",
        "object gained aliased\n",
        "object gained by_define_method\n",
        "object gained by_define_singleton_method\n",
        "Host gained singleton_method_added\n",
        "Host gained class_side\n",
        "1\n",
        "true\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/added.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_added_parens_execution() {
    let expected = concat!(
        "object gained singleton_method_added\n",
        "object gained by_def\n",
        "object gained in_singleton_body\n",
        "object gained aliased\n",
        "object gained by_define_method\n",
        "object gained by_define_singleton_method\n",
        "Host gained singleton_method_added\n",
        "Host gained class_side\n",
        "1\n",
        "true\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/added_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_removed_execution() {
    let expected = concat!(
        "class lost to_remove\n",
        "false\n",
        "object lost gone\n",
        "false\n",
        "NameError\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/removed.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_removed_parens_execution() {
    let expected = concat!(
        "class lost to_remove\n",
        "false\n",
        "object lost gone\n",
        "false\n",
        "NameError\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/removed_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_undefined_execution() {
    let expected = concat!(
        "true\n",
        "class undefined to_undefine\n",
        "false\n",
        "NoMethodError after undef\n",
        "NameError\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/undefined.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_singleton_hooks_undefined_parens_execution() {
    let expected = concat!(
        "true\n",
        "class undefined to_undefine\n",
        "false\n",
        "NoMethodError after undef\n",
        "NameError\n"
    );
    let output = run_example("metaprogramming/singleton_hooks/undefined_parens.rb");
    assert_eq!(output, expected);
}
