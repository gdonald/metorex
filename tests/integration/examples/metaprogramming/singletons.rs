// The class one object alone has.

use super::super::run_example;
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
