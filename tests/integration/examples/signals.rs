use super::run_example;

#[test]
fn test_signals_delivery_execution() {
    let expected = "true\ntrue\ntrue\nInterrupt\nshutting down\ntrue\nSIGTERM\ntrue\nDEFAULT\ntrue\nInterrupt\nSIGTERM\n1\nhandled true\nunsupported signal `SIGNOPE'\n";
    let output = run_example("signals/delivery.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_signals_delivery_parens_execution() {
    let expected = "true\ntrue\ntrue\nInterrupt\nshutting down\ntrue\nSIGTERM\ntrue\nDEFAULT\ntrue\nInterrupt\nSIGTERM\n1\nhandled true\nunsupported signal `SIGNOPE'\n";
    let output = run_example("signals/delivery_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_signals_exception_construction_execution() {
    let expected = "SIGINT\nSIGINT\ntrue\nSIGINT\nSIGINT\nSIGTERM\nSIGTERM\ncustom name\ncustom name\ntrue\ninvalid signal number 100000\ninvalid signal name NONEXISTENT\ninvalid signal name NONEXISTENT\nbad signal type Object\nwrong number of arguments (given 2, expected 1)\nstill a message\ntrue\n";
    let output = run_example("signals/exception_construction.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_signals_exception_construction_parens_execution() {
    let expected = "SIGINT\nSIGINT\ntrue\nSIGINT\nSIGINT\nSIGTERM\nSIGTERM\ncustom name\ncustom name\ntrue\ninvalid signal number 100000\ninvalid signal name NONEXISTENT\ninvalid signal name NONEXISTENT\nbad signal type Object\nwrong number of arguments (given 2, expected 1)\nstill a message\ntrue\n";
    let output = run_example("signals/exception_construction_parens.rb");
    assert_eq!(output, expected);
}
