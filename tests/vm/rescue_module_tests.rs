// A rescue clause naming a module catches an exception whose class includes
// that module, as `rescue IO::WaitWritable` does.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn inspected(code: &str) -> String {
    let source = format!("({code}).inspect");
    let tokens = Lexer::new(&source).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    match vm.execute_program(&statements).expect("execution failed") {
        Some(Object::String(text)) => text.as_str().to_string(),
        other => panic!("expected an inspection, got {other:?}"),
    }
}

#[test]
fn a_module_included_in_the_raised_class_catches_it() {
    assert_eq!(
        inspected(
            "module Tagged; end\nclass Oops < StandardError\n  include Tagged\nend\nbegin\n  raise Oops\nrescue Tagged => error\n  error.class\nend"
        ),
        "Oops"
    );
}

#[test]
fn a_module_included_in_a_superclass_catches_the_subclass() {
    assert_eq!(
        inspected(
            "module Tagged; end\nclass Oops < StandardError\n  include Tagged\nend\nclass Worse < Oops; end\nbegin\n  raise Worse\nrescue Tagged => error\n  error.class\nend"
        ),
        "Worse"
    );
}

#[test]
fn a_module_the_raised_class_lacks_lets_it_pass() {
    assert_eq!(
        inspected(
            "module Tagged; end\nbegin\n  begin\n    raise ArgumentError\n  rescue Tagged\n    :wrong\n  end\nrescue ArgumentError\n  :passed\nend"
        ),
        ":passed"
    );
}

#[test]
fn wait_writable_catches_the_error_a_full_socket_raises() {
    assert_eq!(
        inspected(
            "begin\n  raise IO::EAGAINWaitWritable, 'x'\nrescue IO::WaitWritable => error\n  error.message\nend"
        ),
        "\"Resource temporarily unavailable - x\""
    );
}
