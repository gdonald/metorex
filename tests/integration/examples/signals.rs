use super::run_example;

#[test]
fn test_signals_delivery_execution() {
    let expected = "true\ntrue\ntrue\nInterrupt\nshutting down\ntrue\nSIGTERM\ntrue\nDEFAULT\ntrue\nInterrupt\nSIGTERM\n1\nhandled true\nunsupported signal 'SIGNOPE'\n";
    let output = run_example("signals/delivery.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_signals_delivery_parens_execution() {
    let expected = "true\ntrue\ntrue\nInterrupt\nshutting down\ntrue\nSIGTERM\ntrue\nDEFAULT\ntrue\nInterrupt\nSIGTERM\n1\nhandled true\nunsupported signal 'SIGNOPE'\n";
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

#[test]
fn test_signals_signame_execution() {
    let expected = concat!(
        "EXIT\n",
        "TERM\n",
        "nil\n",
        "ABRT\n",
        "CHLD\n",
        "true\n",
        "EXIT\n",
        "no implicit conversion of String into Integer\n",
        "can't convert NotANumber to Integer (NotANumber#to_int gives String)\n"
    );
    let output = run_example("signals/signame.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_signals_signame_parens_execution() {
    let expected = concat!(
        "EXIT\n",
        "TERM\n",
        "nil\n",
        "ABRT\n",
        "CHLD\n",
        "true\n",
        "EXIT\n",
        "no implicit conversion of String into Integer\n",
        "can't convert NotANumber to Integer (NotANumber#to_int gives String)\n"
    );
    let output = run_example("signals/signame_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `signals/streams_of_a_command` variants, which differ only in
/// whether the calls are written with parentheses.
const STREAMS_OF_A_COMMAND: &str = concat!(
    "\"to_the_output\\nto_the_error\\n\"\n",
    "0\n",
    "\"only_the_output\\n\"\n",
    "\"\"\ntrue\nnil\n"
);

#[test]
fn test_signals_streams_of_a_command_execution() {
    let output = run_example("signals/streams_of_a_command.rb");
    assert_eq!(output, STREAMS_OF_A_COMMAND);
}

#[test]
fn test_signals_streams_of_a_command_no_parens_execution() {
    let output = run_example("signals/streams_of_a_command_no_parens.rb");
    assert_eq!(output, STREAMS_OF_A_COMMAND);
}

/// The expected output of both `signals/what_answers_a_signal` variants, which differ only in
/// whether the calls are written with parentheses.
const WHAT_ANSWERS_A_SIGNAL: &str = concat!(
    "\"IGNORE\"\n",
    "answered true\n",
    "\"SYSTEM_DEFAULT\"\n",
    "true\n",
    "\"DEFAULT\"\n",
    "\"can't trap reserved signal: SIGSEGV\"\n",
    "\"invalid signal number (300)\"\n",
    "\"bad signal type NilClass\"\n",
    "on the way out\n",
    "at_exit\n"
);

#[test]
fn test_signals_what_answers_a_signal_execution() {
    let output = run_example("signals/what_answers_a_signal.rb");
    assert_eq!(output, WHAT_ANSWERS_A_SIGNAL);
}

#[test]
fn test_signals_what_answers_a_signal_no_parens_execution() {
    let output = run_example("signals/what_answers_a_signal_no_parens.rb");
    assert_eq!(output, WHAT_ANSWERS_A_SIGNAL);
}

/// The expected output of both `signals/trapped_from_outside` variants.
const TRAPPED_FROM_OUTSIDE_OUTPUT: &str = concat!("[15]\n", "1\n");

#[test]
fn test_signals_trapped_from_outside_execution() {
    let output = run_example("signals/trapped_from_outside.rb");
    assert_eq!(output, TRAPPED_FROM_OUTSIDE_OUTPUT);
}

#[test]
fn test_signals_trapped_from_outside_no_parens_execution() {
    let output = run_example("signals/trapped_from_outside_no_parens.rb");
    assert_eq!(output, TRAPPED_FROM_OUTSIDE_OUTPUT);
}
