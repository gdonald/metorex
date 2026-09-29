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

/// The expected output of both `metaprogramming/refinement_imports` variants.
const REFINEMENT_IMPORTS_OUTPUT: &str = concat!(
    "   foo.\n",
    "true\n",
    "8\n",
    "5\n",
    "TypeError: wrong argument type Class (expected Module)\n",
    "ArgumentError: Can't import method which is not defined with Ruby code: Kernel#...\n",
    "ArgumentError: Can't import method which is not defined with Ruby code: Zlib#...\n",
    "true\n",
    "<metorex>/zlib.rb\n",
    "has ancestors, but Refinement#import_methods doesn't import their methods\n",
);

#[test]
fn test_metaprogramming_refinement_imports_execution() {
    let output = run_example("metaprogramming/refinement_imports.rb");
    assert_eq!(output, REFINEMENT_IMPORTS_OUTPUT);
}

#[test]
fn test_metaprogramming_refinement_imports_no_parens_execution() {
    let output = run_example("metaprogramming/refinement_imports_no_parens.rb");
    assert_eq!(output, REFINEMENT_IMPORTS_OUTPUT);
}
