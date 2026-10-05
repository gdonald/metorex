// What a refused constant read or write reports.

use super::*;
// ── Dir methods (mod.rs lines 60-116) ────────────────────────────────────

#[test]
fn dir_exist_no_args_errors() {
    let err = run_err("Dir.exist?");
    assert!(err.contains("argument"));
}

#[test]
fn dir_exist_non_string_arg_errors() {
    let err = run_err("Dir.exist?(42)");
    assert!(err.contains("String"));
}

#[test]
fn dir_mkdir_no_args_errors() {
    let err = run_err("Dir.mkdir");
    assert!(err.contains("argument"));
}

#[test]
fn dir_mkdir_non_string_arg_errors() {
    let err = run_err("Dir.mkdir(42)");
    assert!(err.contains("String"));
}

#[test]
fn dir_pwd_returns_string() {
    let result = run("Dir.pwd");
    match result {
        Some(Object::String(s)) => assert!(!s.as_str().is_empty()),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn dir_getwd_returns_string() {
    let result = run("Dir.getwd");
    match result {
        Some(Object::String(s)) => assert!(!s.as_str().is_empty()),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn dir_exist_returns_true_for_tmp() {
    let result = run(r#"Dir.exist?("/tmp")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn dir_exists_returns_false_for_nonexistent() {
    let result = run(r#"Dir.exist?("/this_path_does_not_exist_xyz_123")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn dir_mkdir_and_delete() {
    let path = "/tmp/metorex_dir_test_coverage_xyz";
    let _ = std::fs::remove_dir(path);
    let result = run(&format!(r#"Dir.mkdir("{}")"#, path));
    assert_eq!(result, Some(Object::Int(0)));
    let result2 = run(&format!(r#"Dir.delete("{}")"#, path));
    assert_eq!(result2, Some(Object::Int(0)));
}

#[test]
fn dir_glob_returns_array() {
    let result = run(r#"Dir.glob("/tmp")"#);
    match result {
        Some(Object::Array(_)) => {}
        other => panic!("expected Array, got {:?}", other),
    }
}

// ── Time.now ─────────────────────────────────────────────────────────────

#[test]
fn time_now_answers_a_time() {
    let result = run("Time.now.class.name");
    assert_eq!(result, Some(Object::string("Time")));
}

#[test]
fn time_new_without_arguments_answers_the_current_time() {
    let result = run("Time.new.to_i > 1600000000");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── module_function with Symbol name (mod.rs lines 614-620) ──────────────

#[test]
fn module_function_with_symbol_name_symbol() {
    let result = run(r#"
module MathUtils
  def double(n)
    n * 2
  end
  module_function :double
end
MathUtils.double(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ── Process module stubs (mod.rs lines 669-675) ───────────────────────────

#[test]
fn process_pid_returns_integer() {
    let result = run("Process.pid");
    match result {
        Some(Object::Int(n)) => assert!(n > 0),
        other => panic!("expected Int, got {:?}", other),
    }
}

#[test]
fn process_ppid_returns_integer() {
    // Every process has a parent, so the id is a positive number rather than
    // the zero a stub would answer.
    let result = run("Process.ppid");
    assert!(
        matches!(result, Some(Object::Int(parent)) if parent > 0),
        "unexpected parent process id: {:?}",
        result
    );
}

#[test]
fn process_kill_without_a_signal_errors() {
    let err = run_err("Process.kill");
    assert!(err.contains("wrong number of arguments"));
}

#[test]
fn process_exit_raises_system_exit() {
    // `Process.exit` ends the program the way the bare form does, which is a
    // rescuable SystemExit rather than a nil answer.
    let error = run_err("Process.exit");
    assert!(error.contains("Uncaught exception: exit"), "{}", error);
}

// ── GC / ObjectSpace stubs (mod.rs lines 681-683) ────────────────────────

#[test]
fn gc_start_returns_nil() {
    let result = run("GC.start");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn objectspace_name_it_keeps_no_account_of_returns_nil() {
    let result = run("ObjectSpace.count_nodes");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn objectspace_each_object_without_a_block_returns_an_enumerator() {
    let result = run("ObjectSpace.each_object.class.name");
    assert_eq!(result, Some(Object::string("Enumerator")));
}

#[test]
fn objectspace_each_object_with_no_module_walks_every_object() {
    let result = run("class Widget; end; kept = Widget.new; \
         ObjectSpace.each_object.to_a.any? { |found| found.equal?(kept) }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn objectspace_each_object_leaves_out_the_singleton_class_of_a_singleton_class() {
    let result = run(
        "inner = Class.new.singleton_class; outer = inner.singleton_class; \
         found = ObjectSpace.each_object(Class).to_a; \
         [found.any? { |one| one.equal?(inner) }, found.any? { |one| one.equal?(outer) }]",
    );
    assert_eq!(
        result,
        Some(Object::array(vec![Object::Bool(true), Object::Bool(false)]))
    );
}

// ── File.expand_path with non-existent path (mod.rs line 451) ────────────

#[test]
fn file_expand_path_nonexistent_path() {
    let result = run(r#"File.expand_path("/tmp/nonexistent_xyz_abc/subdir")"#);
    match result {
        Some(Object::String(s)) => assert!(s.as_str().contains("nonexistent_xyz_abc")),
        other => panic!("expected String, got {:?}", other),
    }
}

// ── object_methods.rs: to_s with args error (lines 45-49) ────────────────────

#[test]
fn to_s_with_too_many_args_errors() {
    let err = run_err("42.to_s(10, 2)");
    assert!(err.contains("argument"));
}

// ── object_methods.rs: respond_to? with Symbol arg (line 78) ─────────────────

#[test]
fn respond_to_with_symbol_arg() {
    let result = run(r#"
class Foo
  def bar
    42
  end
end
Foo.new.respond_to?(:bar)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── object_methods.rs: dup/clone on Array (lines 399-402) ────────────────────

#[test]
fn array_dup_returns_copy() {
    let result = run(r#"
arr = [1, 2, 3]
copy = arr.dup
copy.push(4)
arr.length
"#);
    // Original array should still have 3 elements
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_clone_returns_copy() {
    let result = run(r#"
arr = [1, 2, 3]
copy = arr.clone
copy.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Process.setproctitle ─────────────────────────────────────────────────

#[test]
fn process_setproctitle_rewrites_the_arguments_the_process_started_with() {
    let title = "metorex-title-in-process";
    let result = run(&format!("Process.setproctitle({title:?})"));
    assert_eq!(result, Some(Object::string(title)));
    assert_eq!(std::env::args().next().as_deref(), Some(title));
}

// ── Signals and Process.kill ─────────────────────────────────────────────

#[test]
fn process_kill_with_signal_zero_only_checks_the_process() {
    let result = run("Process.kill(0, Process.pid)");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn process_kill_with_a_minus_name_signals_the_process_group() {
    let result = run("child = spawn(\"sleep 5\", pgroup: true); \
         sent = Process.kill(\"-CONT\", child); \
         Process.kill(:TERM, child); \
         Process.wait(child); \
         sent");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn process_kill_refuses_exit_by_name() {
    let error = run_err("Process.kill(\"-EXIT\", Process.pid)");
    assert!(error.contains("unsupported signal 'SIGEXIT'"), "{error}");
}

#[test]
fn process_kill_reads_a_pid_through_to_int() {
    let result = run("pid = Object.new; def pid.to_int = Process.pid; Process.kill(0, pid)");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn signal_trap_runs_its_handler_when_another_process_sends_the_signal() {
    let result = run("received = nil; \
         Signal.trap(:USR2) { |number| received = number }; \
         system(\"kill -USR2 #{Process.pid}\"); \
         sleep(0.01) while received.nil?; \
         Signal.trap(:USR2, \"IGNORE\"); \
         Signal.trap(:USR2, \"DEFAULT\"); \
         received");
    assert_eq!(result, Some(Object::Int(libc::SIGUSR2 as i64)));
}

#[test]
fn signal_trap_leaves_a_signal_it_never_caught_to_the_system() {
    let result = run("Signal.trap(:USR1, \"IGNORE\")");
    assert_eq!(result, Some(Object::string("SYSTEM_DEFAULT")));
}

#[test]
fn rbconfig_ruby_names_the_binary_running_the_program() {
    let result = run("require \"rbconfig\"; RbConfig.ruby");
    let binary = std::env::current_exe().expect("current_exe");
    assert_eq!(result, Some(Object::string(binary.display().to_string())));
}

#[test]
fn a_thread_reading_a_pipe_lets_the_main_thread_write_to_it() {
    let result = run("reader, writer = IO.pipe; \
         waiting = Thread.new { reader.read(5) }; \
         Thread.pass; \
         writer.write(\"hello\"); \
         writer.close; \
         waiting.value");
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn a_signal_caught_on_one_thread_is_handled_there_and_not_where_it_arrives() {
    let (caught_tx, caught_rx) = std::sync::mpsc::channel();
    let (sent_tx, sent_rx) = std::sync::mpsc::channel();
    let catcher = std::thread::spawn(move || {
        let program = |code: &str| {
            Parser::new(Lexer::new(code).tokenize())
                .parse()
                .expect("parse failed")
        };
        let mut vm = VirtualMachine::new();
        vm.execute_program(&program(
            "$winched = 0; Signal.trap(:WINCH) { $winched += 1 }",
        ))
        .expect("trap failed");
        caught_tx.send(()).expect("send");
        sent_rx.recv().expect("recv");
        let handled = vm
            .execute_program(&program("x = 1; $winched"))
            .expect("handling failed");
        vm.execute_program(&program("Signal.trap(:WINCH, \"SYSTEM_DEFAULT\")"))
            .expect("restoring failed");
        match handled {
            Some(Object::Int(count)) => count,
            other => panic!("expected a count, got {other:?}"),
        }
    });
    caught_rx.recv().expect("recv");
    // SAFETY: SIGWINCH arrives on this thread, and the handler the thread
    // above installed has run by the time `raise` returns.
    unsafe { libc::raise(libc::SIGWINCH) };

    let here = run("x = 1; x + 1");
    sent_tx.send(()).expect("send");
    let there = catcher.join().expect("join");

    assert_eq!((here, there), (Some(Object::Int(2)), 1));
}

// ── Process.exec and Process.spawn ───────────────────────────────────────

#[test]
fn process_exec_of_an_empty_command_raises_enoent() {
    let error = run_err("Process.exec(\"\")");
    assert!(error.contains("No such file or directory"), "{}", error);
}

#[test]
fn process_exec_of_a_directory_raises_eacces() {
    let error = run_err("Process.exec(\"/\")");
    assert!(error.contains("Permission denied"), "{}", error);
}

#[test]
fn process_exec_refuses_a_null_byte() {
    let error = run_err("Process.exec(\"echo\\0\")");
    assert!(error.contains("string contains null byte"), "{}", error);
}

#[test]
fn process_exec_refuses_a_command_array_that_is_not_a_pair() {
    let error = run_err("Process.exec([\"/bin/sh\"], \"-c\", \"true\")");
    assert!(error.contains("wrong first argument"), "{}", error);
}

#[test]
fn process_spawn_reads_the_environment_and_the_command_through_to_hash_and_to_ary() {
    let result = run("environment = Object.new; \
         def environment.to_hash = { \"SPAWNED\" => \"yes\" }; \
         command = Object.new; \
         def command.to_ary = [\"/bin/sh\", \"named\"]; \
         pid = Process.spawn(environment, command, \"-c\", \"test \\\"$0 $SPAWNED\\\" = \\\"named yes\\\"\", unsetenv_others: true); \
         Process.wait(pid); \
         $?.success?");
    assert_eq!(result, Some(Object::Bool(true)));
}
