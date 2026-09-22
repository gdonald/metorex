// Handles, directories, thread groups and the calls that reach the
// operating system.

use super::*;

#[test]
fn a_closed_handle_refuses_to_be_read() {
    let error = run_err(
        r#"
path = "/tmp/metorex_closed_handle_test.txt"
File.write(path, "abc\n")
handle = File.open(path)
handle.close
begin
  handle.getc
ensure
  File.delete(path)
end
"#,
    );
    assert!(
        error.contains("closed stream"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_closed_handle_still_says_what_it_was_opened_on() {
    let result = run(r#"
path = "/tmp/metorex_closed_handle_path.txt"
File.write(path, "abc\n")
handle = File.open(path)
handle.close
answer = [handle.closed?, handle.path, handle.to_io.equal?(handle)]
File.delete(path)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, /tmp/metorex_closed_handle_path.txt, true]".to_string())
    );
}

#[test]
fn reading_a_directory_is_refused_as_a_directory() {
    let error = run_err(r#"File.read("/tmp")"#);
    assert!(
        error.contains("Is a directory"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_missing_name_has_no_link_to_read() {
    let error = run_err(r#"File.readlink("/tmp/metorex_no_such_link_here")"#);
    assert!(
        error.contains("No such file or directory"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_thread_belongs_to_the_default_group_until_another_takes_it() {
    let result = run(r#"
group = ThreadGroup.new
before = Thread.main.group.equal?(ThreadGroup::Default)
group.add(Thread.main)
[before, Thread.main.group.equal?(group), group.list.include?(Thread.main)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true, true]".to_string())
    );
}

#[test]
fn an_enclosed_group_refuses_to_give_a_thread_up() {
    let error = run_err(
        r#"
held = ThreadGroup.new
held.add(Thread.main)
held.enclose
ThreadGroup.new.add(Thread.main)
"#,
    );
    assert!(
        error.contains("enclosed thread group"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn a_refinement_refuses_to_have_a_module_mixed_into_it() {
    let error = run_err(
        r#"
Module.new do
  refine String do
    include Module.new
  end
end
"#,
    );
    assert!(
        error.contains("Refinement#include has been removed"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn syscall_and_set_trace_func_are_private_kernel_methods() {
    let result = run(r#"
[Kernel.private_instance_methods(false).include?(:syscall),
 Kernel.private_instance_methods(false).include?(:set_trace_func)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true]".to_string())
    );
}

#[test]
fn a_hash_subclass_keeps_its_class_through_merge() {
    let result = run(r#"
class MergeKeepsClass < Hash; end
held = MergeKeepsClass.new
held[1] = 2
merged = held.merge({ 3 => 4 })
[merged.class.name, merged[1], merged[3]]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[MergeKeepsClass, 2, 4]".to_string())
    );
}

#[test]
fn coerce_refuses_a_string_and_a_numeric_that_is_not_real() {
    let refused_string = run_err(r#"Complex(1, 0).coerce("20")"#);
    assert!(
        refused_string.contains("can't be coerced into Complex"),
        "unexpected error: {}",
        refused_string
    );
}

#[test]
fn mkfifo_reads_a_name_through_to_path() {
    let result = run(r#"
class FifoName
  def initialize(path)
    @path = path
  end
  def to_path
    @path
  end
end
path = "/tmp/metorex_fifo_to_path"
File.delete(path) if File.exist?(path)
File.mkfifo(FifoName.new(path))
answer = File.pipe?(path)
File.delete(path)
answer
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn mkfifo_refuses_a_name_it_cannot_read() {
    let error = run_err(r#"File.mkfifo(:"/tmp/metorex_fifo_symbol")"#);
    assert!(error.contains("String"), "unexpected error: {}", error);
}

#[test]
fn mkfifo_reports_a_directory_that_is_not_there() {
    let error = run_err(r#"File.mkfifo("/metorex_no_such_directory/fifo")"#);
    assert!(
        error.contains("No such file or directory"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn mkfifo_reports_a_directory_it_may_not_write() {
    // A directory with no write bit refuses the name, which is a different
    // refusal from a missing directory.
    let error = run_err(
        r#"
holder = "/tmp/metorex_fifo_unwritable"
Dir.mkdir(holder) unless Dir.exist?(holder)
File.chmod(0555, holder)
begin
  File.mkfifo(holder + "/fifo")
ensure
  File.chmod(0755, holder)
  Dir.rmdir(holder)
end
"#,
    );
    assert!(
        error.contains("Permission denied"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn coerce_refuses_a_numeric_that_says_it_is_not_real() {
    let error = run_err(
        r#"
class NotReal < Numeric
  def real?
    false
  end
end
Complex(1, 0).coerce(NotReal.new)
"#,
    );
    assert!(
        error.contains("can't be coerced into Complex"),
        "unexpected error: {}",
        error
    );
}
