use super::run_example;

/// The expected output of both `fibers/listing_methods` variants.
const LISTING_METHODS_OUTPUT: &str = concat!("true\n", "Fiber\n");

#[test]
fn test_fibers_listing_methods_execution() {
    let output = run_example("fibers/listing_methods.rb");
    assert_eq!(output, LISTING_METHODS_OUTPUT);
}

#[test]
fn test_fibers_listing_methods_no_parens_execution() {
    let output = run_example("fibers/listing_methods_no_parens.rb");
    assert_eq!(output, LISTING_METHODS_OUTPUT);
}

/// The expected output of both `fibers/transferring` variants.
const TRANSFERRING_OUTPUT: &str = concat!(
    ":first\n",
    ":second\n",
    ":finisher\n",
    "\"attempt to transfer to a yielding fiber\"\n",
    "[1, 2]\n",
    ":back\n",
    "\"dead fiber called\"\n",
    "\"attempt to transfer to a resuming fiber\"\n",
    "\"attempt to transfer to a resuming fiber\"\n",
    "\"fiber called across threads\"\n",
    "42\n",
    ":from_helper\n",
    ":helper_done\n",
    ":up\n",
    "[true, true]\n",
    "\"attempt to transfer to a resuming fiber\"\n",
    "[[:down, :low_done], :high_done]\n",
    "[false, false]\n",
);

#[test]
fn test_fibers_transferring_execution() {
    let output = run_example("fibers/transferring.rb");
    assert_eq!(output, TRANSFERRING_OUTPUT);
}

#[test]
fn test_fibers_transferring_no_parens_execution() {
    let output = run_example("fibers/transferring_no_parens.rb");
    assert_eq!(output, TRANSFERRING_OUTPUT);
}

/// The expected output of both `fibers/raising` variants.
const RAISING_OUTPUT: &str = concat!(
    "\"cannot raise exception on unborn fiber\"\n",
    "\"attempt to resume a terminated fiber\"\n",
    "[:rescued, \"into the transfer\"]\n",
    "\"through the resumer\"\n",
    "[false, false]\n",
    "\"back up\"\n",
    "\"fiber called across threads\"\n",
    ":resumed\n",
);

#[test]
fn test_fibers_raising_execution() {
    let output = run_example("fibers/raising.rb");
    assert_eq!(output, RAISING_OUTPUT);
}

#[test]
fn test_fibers_raising_no_parens_execution() {
    let output = run_example("fibers/raising_no_parens.rb");
    assert_eq!(output, RAISING_OUTPUT);
}
