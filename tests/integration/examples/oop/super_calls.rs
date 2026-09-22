// Reaching the method a class inherits.

use super::super::run_example;
#[test]
fn test_oop_super_basic_execution() {
    let expected = "Buddy\nGolden Retriever\nSome sound -> Woof!\nI am an animal named Buddy\n";
    let output = run_example("oop/super/basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_chain_basic_execution() {
    let expected = "GrandParent\nParent\nChild\n";
    let output = run_example("oop/super/chain_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_keyword_execution() {
    let expected = "Rex makes a sound\nRex barks\nAnimal: Rex, Breed: Labrador\n";
    let output = run_example("oop/super/keyword.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_keyword_parens_execution() {
    let expected = "Rex makes a sound\nRex barks\nAnimal: Rex, Breed: Labrador\n";
    let output = run_example("oop/super/keyword_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_reaches_kernel() {
    let expected = concat!(
        "false\n:ready\ntrue\ntrue\n",
        "super: no superclass method 'no_such_kernel_method' for an instance of Missing\n"
    );
    let output = run_example("oop/super_reaches_kernel.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_reaches_kernel_no_parens() {
    let expected = concat!(
        "false\n:ready\ntrue\ntrue\n",
        "super: no superclass method 'no_such_kernel_method' for an instance of Missing\n"
    );
    let output = run_example("oop/super_reaches_kernel_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_forms_execution() {
    let expected = concat!(
        "[:nested]\n",
        "[:explicit]\n",
        "true\n",
        "[:archived]\n",
        "15\n",
    );
    let output = run_example("oop/super_forms.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_forms_parens_execution() {
    let expected = concat!(
        "[:nested]\n",
        "[:explicit]\n",
        "true\n",
        "[:archived]\n",
        "15\n",
    );
    let output = run_example("oop/super_forms_parens.rb");
    assert_eq!(output, expected);
}
