// A constant loaded while several threads ask for it: the thread that gets
// there first loads the file and the rest wait for it, so none of them reads
// a module that is only part-way built.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&statements).expect("execution failed")
}

/// The race, written against the fixture the examples load, with the number of
/// names and threads given.
fn racing_source(names: &str, hands: usize) -> String {
    let fixture = format!(
        "{}/tests/_examples/threads/fixtures/counted_autoload.rb",
        env!("CARGO_MANIFEST_DIR")
    );
    format!(
        r#"
class Counter
  def initialize
    @value = 0
    @guard = Mutex.new
  end
  def get
    @guard.synchronize {{ @value }}
  end
  def increment_and_get
    @guard.synchronize do
      @value += 1
      @value
    end
  end
end

class Gate
  def initialize(count)
    @count = count
    @waiting = 0
    @guard = Mutex.new
    @gate = ConditionVariable.new
  end
  def await
    @guard.synchronize do
      @waiting += 1
      if @waiting >= @count
        @waiting = 0
        @gate.broadcast
        true
      else
        @gate.wait @guard
        false
      end
    end
  end
end

$counted_loads = Counter.new
file = "{fixture}"
named = file.sub(/\.rb\Z/, "")
names = [{names}]
names.each {{ |name| Object.autoload(name, named) }}

gate = Gate.new({hands})
threads = (1..{hands}).map do
  Thread.new do
    names.map do |name|
      last_one_in = gate.await
      $LOADED_FEATURES.delete(file) if last_one_in && $LOADED_FEATURES.include?(file)
      gate.await
      Object.const_get(name).ready
    end
  end
end
answers = threads.map {{ |thread| thread.value }}
[answers.flatten.all? {{ |answered| answered == :ready }}, $counted_loads.get]
"#
    )
}

#[test]
fn every_thread_reads_a_module_the_load_finished_building() {
    let result = run(&racing_source(":Counted1, :Counted2", 4));
    let Some(Object::Array(answered)) = result else {
        panic!("Got: {:?}", result)
    };
    assert_eq!(answered.borrow()[0], Object::Bool(true));
}

#[test]
fn the_file_is_loaded_once_for_each_name_it_was_registered_under() {
    let result = run(&racing_source(":Counted1, :Counted2, :Counted3", 3));
    let Some(Object::Array(answered)) = result else {
        panic!("Got: {:?}", result)
    };
    assert_eq!(answered.borrow()[1], Object::Int(3));
}

#[test]
fn a_method_that_ran_threads_still_returns_from_itself() {
    let result = run(r#"
def counted
  threads = (1..3).map { |n| Thread.new { n } }
  threads.each { |t| t.value }
  begin
    return :from_the_method
  ensure
    nil
  end
end
counted
"#);
    assert_eq!(result, Some(Object::symbol("from_the_method".to_string())));
}
