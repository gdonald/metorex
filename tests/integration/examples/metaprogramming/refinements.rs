// Methods brought into scope for one file only.

use super::super::run_example;
#[test]
fn test_metaprogramming_refinement_execution() {
    let output = run_example("metaprogramming/refinement.rb");
    assert_eq!(output, "HELLO!\n");
}

#[test]
fn test_metaprogramming_refine_block_scope_execution() {
    let expected = concat!(
        "HELLO\n",
        "int 1, int 2\n",
        "no block given\n",
        "wrong argument type String (expected Class or Module)\n",
        "refined a module\n",
    );
    let output = run_example("metaprogramming/refine_block_scope.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_refine_block_scope_parens_execution() {
    let expected = concat!(
        "HELLO\n",
        "int 1, int 2\n",
        "no block given\n",
        "wrong argument type String (expected Class or Module)\n",
        "refined a module\n",
    );
    let output = run_example("metaprogramming/refine_block_scope_parens.rb");
    assert_eq!(output, expected);
}
