# Ruby Roadmap

Every spec file in `ruby/spec` passes in `scripts/run_ruby_spec.sh`, but
ruby/spec does not cover all of Ruby. This roadmap lists what metorex still
lacks or does differently from MRI 4.0, measured against MRI 4.0.6 on the same
machine. Each item names the evidence for the gap and the acceptance criterion
for calling it done. Every item adds a `tests/_examples` file with a
`_no_parens` variant, wired into `tests/integration/examples/`, whose output
matches MRI's.

## 1. Ractor

`defined?(Ractor)` answers nil in metorex. ruby/spec has no Ractor specs, so
the acceptance criteria below are checked against MRI's output.

- [x] 1.1. The Ractor class and the main Ractor
  - [x] 1.1.1. `Ractor.current`, `Ractor.main`, `Ractor.main?`, `Ractor.count`, `Ractor#name` and `Ractor#inspect`
  - [x] 1.1.2. `Ractor.new` running its block, with the arguments it is given copied in and `self` inside the block being the new Ractor
  - [x] 1.1.3. `Ractor#value` and `Ractor#join` waiting for the block to end, and an exception that ended it raised as `Ractor::RemoteError` with the original as `cause`
- [x] 1.2. Isolation between Ractors
  - [x] 1.2.1. A block passed to `Ractor.new` that reads an outer local is refused with ArgumentError when it is made, as MRI's isolated Proc check does
  - [x] 1.2.2. Reading a global variable, a class variable or a non-shareable constant from a non-main Ractor raises `Ractor::IsolationError`
  - [x] 1.2.3. `Ractor.make_shareable` deep-freezing an object graph, `Ractor.shareable?`, and `Ractor.make_shareable(proc)` for a Proc whose `self` and captures are shareable, with `Ractor.shareable_proc` and `Ractor.shareable_lambda`
- [x] 1.3. Message passing
  - [x] 1.3.1. `Ractor::Port` with `send`/`<<`, `receive`, `close` and `closed?`, and `Ractor.receive` and `Ractor#send` on the default port
  - [x] 1.3.2. `send(obj, move: true)` moving an object so the sender's reference raises `Ractor::MovedError` when used
  - [x] 1.3.3. `Ractor.select` over ports, answering the port and the value
- [x] 1.4. Ractor-local storage: `Ractor#[]`, `Ractor#[]=` and `Ractor.store_if_absent`

## 2. RubyVM

`RubyVM` is an uninitialized constant in metorex. MRI defines it, and tools
and gems test for it.

- [x] 2.1. `RubyVM` itself: `RubyVM::OPTS`, `RubyVM::INSTRUCTION_NAMES`, `RubyVM.stat` with the keys MRI answers, and `RubyVM.keep_script_lines` and its writer
- [x] 2.2. `RubyVM::InstructionSequence` with `compile`, `compile_file`, `of`, `#eval`, `#path`, `#label`, `#first_lineno` and `#disasm`. Metorex has no bytecode, so `#disasm` and `#to_a` answer a listing built from the AST, and the item records which methods answer differently from MRI. These answer as MRI's do under `--parser=parse.y`: `compile` and `new` with a file, path and line, `compile_file`, `of` for a Proc or a Method with its `block (2 levels) in name` labels, `#eval` and its refusal for a method or block, `#path`, `#absolute_path`, `#label`, `#base_label`, `#first_lineno`, `#inspect`, `#each_child`, `#trace_points` with a line event per statement and the `call` and `return` or `b_call` and `b_return` around a method or block, `compile_option`, `allocate`'s refusal, and SyntaxError messages headed by the file and line. These answer differently: `#disasm` lists the syntax tree's nodes under MRI's heading line, with no local table and no YARV instructions; `#to_a` carries those nodes as its instructions, `parser: :metorex` and empty `node_ids` in its details, no hidden keyword slot among its locals, and no `opt`, `block_start` or `kwbits` among its parameters; and `#to_binary` writes a format only metorex's `load_from_binary` reads
- [x] 2.3. `RubyVM::AbstractSyntaxTree.parse`, `parse_file` and `of`, answering `Node` objects with `type`, `children`, `first_lineno`, `first_column`, `last_lineno` and `last_column`, matching MRI's node types for the forms metorex parses. Built from the tree metorex's Ripper grammar parses, with each node's span taken from the tokens the grammar consumed for it, since the Rust AST records where a node starts and not where it ends. 2.2 lists its instructions from these nodes, so this item comes first. Each sub-item is checked by dumping type, span and children for a corpus of snippets and comparing with MRI's dump
  - [x] 2.3.1. The Ripper grammar records the first and last token of every event it builds, `RubyVM::AbstractSyntaxTree.parse` and `parse_file` answer a `Node` for a program, and `Node#type`, `#children`, `#first_lineno`, `#first_column`, `#last_lineno`, `#last_column` and `#inspect` answer as MRI's do, with `SCOPE`, `BLOCK` and the literal nodes (`INTEGER`, `FLOAT`, `RATIONAL`, `IMAGINARY`, `STR`, `SYM`, `NIL`, `TRUE`, `FALSE`, `SELF`)
  - [x] 2.3.2. Variables, constants, assignments and calls: `LVAR`, `DVAR`, `IVAR`, `GVAR`, `CVAR`, `CONST`, `COLON2`, `COLON3`, `LASGN`, `DASGN`, `IASGN`, `GASGN`, `CVASGN`, `CDECL`, `CALL`, `OPCALL`, `FCALL`, `VCALL`, `QCALL`, `LIST`, `ZLIST`, `HASH`, `SPLAT`, `BLOCK_PASS`, `ATTRASGN`, `OP_ASGN1`, `OP_ASGN2`, `OP_ASGN_OR`, `OP_ASGN_AND`, `OP_CDECL`, `MASGN`
  - [x] 2.3.3. Control flow: `IF`, `UNLESS`, `WHILE`, `UNTIL`, `CASE`, `CASE2`, `CASE3`, `WHEN`, `IN`, `AND`, `OR`, `BEGIN`, `RESCUE`, `RESBODY`, `ENSURE`, `ERRINFO`, `BREAK`, `NEXT`, `REDO`, `RETRY`, `RETURN`, `FOR`, `FLIP2`, `FLIP3`, `DOT2`, `DOT3`
  - [x] 2.3.4. Definitions and blocks: `DEFN`, `DEFS`, `ARGS`, `OPT_ARG`, `KW_ARG`, `CLASS`, `MODULE`, `SCLASS`, `ITER`, `LAMBDA`, `YIELD`, `SUPER`, `ZSUPER`, `ALIAS`, `VALIAS`, `UNDEF`, `DEFINED`
  - [x] 2.3.5. Strings, regexps and patterns: `DSTR`, `EVSTR`, `XSTR`, `DXSTR`, `DSYM`, `REGX`, `DREGX`, `MATCH`, `MATCH2`, `MATCH3`, `ONCE`, `ARYPTN`, `HSHPTN`, `FNDPTN`, `PREEXE`, `POSTEXE`, and the remaining node types MRI answers for a form metorex parses
  - [x] 2.3.6. `RubyVM::AbstractSyntaxTree.of` for a Proc, a Method and a backtrace location, `Node#source`, `#script_lines` with `keep_script_lines: true`, `#tokens` and `#all_tokens`, and `RubyVM::AbstractSyntaxTree.node_id_for_backtrace_location`, and a `SyntaxError` message that quotes the offending line with a caret beneath it, as MRI's `Invalid break\nbreak if a\n^~~~~\n` does
- [x] 2.4. `RubyVM::YJIT` answering `enabled?` false and `enable` as a no-op, so code that checks for it runs
- [ ] 2.5. `RubyVM::AbstractSyntaxTree::Node#locations`, which MRI 4.0 answers as `RubyVM::AbstractSyntaxTree::Location` objects: the node's own span first, then the spans of the keywords and operators MRI records for its type, such as an IF's `if`, `then` and `end`. Acceptance: `locations` answers what MRI's does for each node type the converter builds, checked over the syntax tree corpora

## 3. Threads

Ruby threads run as fibers on one OS thread and take turns. A thread hands
the turn over when it waits, and at the next statement once it has held
the turn for 100 ms.

- [x] 3.1. Hand the turn over by time rather than by statement count. MRI's timer marks a thread's turn as spent every 100 ms, and the thread hands it over at its next method call, block call or loop iteration. Metorex hands it over after 10,000 statements however long they take, so a loop whose statements each take tens of milliseconds keeps the turn for minutes. Neither switches inside one native call: in MRI a counter thread does not advance while the main thread runs a single `big.sort`, and does while it runs `big.sort { |a, b| a <=> b }`. Acceptance: a counter thread advances while the main thread runs a few statements that each take longer than a time slice, checked by the counter's value rather than by a clock
- [x] 3.2. Report a deadlock as MRI does. `Mutex#lock`, `Queue#pop`, `ConditionVariable#wait` and `sleep` with no length give up after 2 seconds and carry on. With a lock held and the holder waiting forever, `t = Thread.new { m.lock; :got }; t.value` answers `:got` in metorex, while MRI raises `fatal` with `No live threads left. Deadlock?` and a listing of each thread. Acceptance: that program raises `fatal` with MRI's message, and no wait has a fixed time limit
- [x] 3.3. Every blocking call hands turns while it waits. `system`, backticks, `Process.wait`, pipe reads, socket reads, `IO.select`, `File#flock` and `sleep` already do, and a C extension's `rb_thread_call_without_gvl` runs on an OS thread of its own. Acceptance: an audit of `src/vm` lists each Kernel, IO, File, Socket and Process method that blocks the interpreter while it waits, and each one on the list hands turns, with an example per method
- [x] 3.4. `Thread#native_thread_id` answering the OS thread id metorex runs on, and the `RUBY_MAX_CPU` and M:N scheduling settings accepted without error

## 4. Standard library and gems

Each name below raises LoadError in metorex and loads in MRI 4.0.6. The list
comes from requiring every file in MRI's `rubylibdir` and every default and
bundled gem one at a time.

- [ ] 4.1. Default libraries
  - [x] 4.1.1. `forwardable`: `def_delegator`, `def_delegators`, `delegate` and `instance_delegate`, with `SingleForwardable`
  - [ ] 4.1.2. `tsort`: `tsort`, `each_strongly_connected_component`, `strongly_connected_components` and `TSort::Cyclic`
  - [ ] 4.1.3. `io/wait`: the require succeeds, and `IO#wait`, `wait_readable`, `wait_writable` and `ready?` answer as MRI's do
  - [ ] 4.1.4. `pty`: `PTY.spawn`, `PTY.open` and `PTY.check`
  - [ ] 4.1.5. `continuation`: `Kernel#callcc` and `Continuation#call`, at least for a continuation called while its frame is still on the stack
  - [ ] 4.1.6. `digest/rmd160`: `Digest::RMD160`
  - [ ] 4.1.7. `pstore`: `PStore` with `transaction`, `[]`, `[]=`, `delete`, `roots`, `abort` and `commit`
  - [ ] 4.1.8. `benchmark`: `Benchmark.measure`, `realtime`, `bm` and `bmbm`
  - [ ] 4.1.9. `un`: `ruby -run -e cp` and the rest of the commands it defines
  - [ ] 4.1.10. `did_you_mean`: suggestions appended to NameError, NoMethodError and KeyError messages, as MRI prints them by default
  - [ ] 4.1.11. `error_highlight`: the `^^^^` line under the failing call in an uncaught error report, as MRI prints it by default
  - [ ] 4.1.12. `syntax_suggest`: the report MRI adds to a SyntaxError about a missing `end`
  - [ ] 4.1.13. `prism`: `Prism.parse`, `Prism.parse_file` and `Prism.lex` answering Prism's node and token classes
  - [ ] 4.1.14. `nkf`, `racc` (the runtime parser `racc/parser`), `mutex_m`, `resolv-replace` and `rinda`
  - [ ] 4.1.15. `reline` and `repl_type_completor`, so `irb` runs interactively
- [ ] 4.2. Bundled gems as MRI ships them: `minitest`, `test/unit`, `rake`, `rexml`, `rss`, `net-smtp`, `net-pop`, `net-imap`, `rbs`, `typeprof`, `debug` and `rdoc`
- [ ] 4.3. Installed gems. `require "rake"` fails although MRI's gem directory holds rake. Acceptance: `Gem.paths` answers MRI's `GEM_HOME` and `GEM_PATH`, `require` of a gem installed there activates it, and `gem` with a version requirement picks that version
- [ ] 4.4. Bundler: `require "bundler/setup"` reading a Gemfile.lock and putting the locked gems on the load path

## 5. Parsing

- [ ] 5.1. Code inside `#{}` reads the locals of the scope it is written in. With a method `foo` defined, `foo = [10, 20]; "#{foo [1]}"` answers `"[:method, [[1]]]"` in metorex and `"20"` in MRI, because the interpolation is parsed by a nested parser that knows no outer bindings
- [ ] 5.2. Code given to `eval`, `instance_eval`, `class_eval` and `Binding#eval` reads the caller's locals while it is parsed. In the same program, `eval("foo [1]")` answers `[:method, [[1]]]` in metorex and `20` in MRI
- [ ] 5.3. Bind locals at parse time rather than from a token pre-scan. The parser decides local or method from a list of binding tokens collected before parsing, and a form the pre-scan does not recognize as a binding reads as a method call. Acceptance: every binding form in the `language/` specs (multiple assignment with splats and nesting, `rescue => var`, `for` variables, block-local `|;x|`, numbered and `it` parameters, pattern `^pin` and alternatives) is declared by the parser where it parses it, and `collect_bound_names` is removed
- [ ] 5.4. The Ripper grammar accepts a brace block after a command's literal argument. `Ripper.sexp("foo 1 { }")` answers a tree in metorex and nil in MRI, where `{` after `1` cannot open a block. Acceptance: the scanner gives that `{` the token MRI's lexer does, and `Ripper.sexp` answers nil for it, at statement level and as `p(foo 1 { })`
- [ ] 5.5. Seven example files are not valid Ruby by `ruby -c`, though metorex runs them: `advanced/stabby_expr_body.rb`, `c_extensions/c_arrays_no_parens.rb`, `c_extensions/c_exceptions_no_parens.rb`, `functions/test_lambdas.rb`, `metaprogramming/caller_locations_slicing.rb`, `runtime/error_reporting.rb` and `runtime/pattern_matching.rb` under `tests/_examples`. Acceptance: each is rewritten as valid Ruby with the same output, and metorex's parser raises SyntaxError for each form MRI rejects, with an example per form
- [ ] 5.6. A syntax error names the token it met the way MRI's parser does. `RubyVM::AbstractSyntaxTree.parse("`a` `b`")` raises `syntax error, unexpected '`'` in metorex and `syntax error, unexpected backtick literal, expecting end-of-input` in MRI, and `case a; in @b; end` raises `unexpected '@b'` against MRI's `unexpected instance variable`. Acceptance: the Ripper grammar's `parse_error` messages name each token kind as MRI's do and list what was expected, checked against MRI over a corpus of invalid programs
- [ ] 5.7. The Ripper grammar reports errors through the events MRI's Ripper uses and finds the ones it misses. Metorex sends every error to `compile_error`, where MRI dispatches `parse_error` for `Invalid return in class/module body` and `class definition in method body`, `assign_error` for `dynamic constant assignment` and `alias_error` for `can't make alias for the number variables`. It does not report `duplicated argument name` for `def f(a, a); end`, `foo { |a, a| }` and `->(a, a) {}`, nor `duplicated variable name` for `in [x, x]`. It names `case a; in x | 1; end` `illegal variable in alternative pattern (x)` where MRI says `alternative pattern after variable capture`, and its numbered parameter and `it` errors lack MRI's second line quoting the earlier use. Acceptance: a Ripper subclass sees the events and messages MRI's does for each of these, and `RubyVM::AbstractSyntaxTree.parse` raises MRI's message with its quoted line

## 6. Behavior that differs from MRI

- [ ] 6.1. The error `String#encode` raises carries what MRI's does. `"\x82\xA0".force_encoding("Shift_JIS").encode("ISO-8859-1")` raises an object that names itself `Encoding::UndefinedConversionError` but has no `source_encoding_name`, `destination_encoding_name`, `source_encoding`, `destination_encoding` or `error_char`, and its message is `U+3042 from UTF-8 to ISO-8859-1` where MRI's is `U+3042 to ISO-8859-1 in conversion from Shift_JIS to UTF-8 to ISO-8859-1`. The same goes for `InvalidByteSequenceError` with its `error_bytes`, `readagain_bytes` and `incomplete_input?`
- [ ] 6.2. `Encoding#names` answers what MRI's does. Metorex adds the constant spellings: `Shift_JIS` answers `["Shift_JIS", "SHIFT_JIS"]` (MRI `["Shift_JIS"]`), `ISO-2022-JP` answers `["ISO-2022-JP", "ISO2022_JP"]` (MRI `["ISO-2022-JP", "ISO2022-JP"]`), and `UTF-7` lacks MRI's `CP65000`. Acceptance: `Encoding.name_list` and every encoding's `names` match MRI's
- [ ] 6.3. `require "ipaddr"` before `require "socket"` makes the second fail with `TypeError: Socket is not a class`, because metorex's `ipaddr.rb` defines `Socket` as a module when it is not yet defined. MRI's `ipaddr` requires `socket`. Acceptance: the two requires work in either order
- [ ] 6.4. `Timeout.timeout` interrupts only `sleep` in metorex. MRI's timeout thread raises into the block wherever it stands, so a busy loop is interrupted too. Acceptance: `Timeout.timeout(0.1) { loop { } }` raises `Timeout::Error`
- [ ] 6.5. `Symbol#inspect` and `Regexp#inspect` of non-UTF-8 sources match MRI, as `String#inspect` of dummy-encoding and binary strings now does
- [ ] 6.6. An error a native method raises inside a thread carries the backtrace of where it was raised. `t = Thread.new { Integer("x") }` reports `<internal:prelude>:11809:in 'Thread#__report_terminated__'` as the raise site, and `t.join` rescued answers a backtrace of `["thread_bt.rb:1:in '<main>'"]`, where MRI answers `["thread_bt.rb:1:in 'Kernel#Integer'", "thread_bt.rb:1:in 'block in <main>'"]`. A Ractor's `RemoteError#cause` carries `<internal:prelude>` lines for the same reason. An error raised by core library code written in Ruby lists that code's `<internal:prelude>` frames, and names the caller's file as `<internal:prelude>`: an uncaught `Ractor.make_shareable` refusal reports `<internal:prelude>:64:in '<main>'` where MRI reports `isolating_a_ractor.rb:64:in 'Ractor.make_shareable'`. Acceptance: these backtraces match MRI's
- [ ] 6.7. `pp` breaks a value across lines at 80 columns the way MRI's does. `pp Ripper.sexp("x = 1 + 2\nputs x")` prints the whole tree on one line in metorex, where MRI prints it over several lines indented by nesting. Acceptance: `pp` output of nested Arrays, Hashes and objects with instance variables matches MRI's for values wider than 80 columns
- [ ] 6.8. An error a native method raises lists every frame between the raise and the rescue. `def calls_missing = nil.no_such_method` called from a block answers a backtrace of `["bt.rb:2:in 'block in <main>'", "bt.rb:4:in '<main>'"]`, where MRI answers `["bt.rb:2:in 'Object#calls_missing'", "bt.rb:4:in 'block in <main>'", "bt.rb:5:in 'block in <main>'", "bt.rb:4:in 'Array#each'", "bt.rb:4:in '<main>'"]`. A `raise` keeps the Ruby frames but lacks native ones such as `Array#each`, `Kernel#Integer` and `Integer#/`. Acceptance: these backtraces match MRI's, and `RubyVM::AbstractSyntaxTree.of` finds the call node for each of their locations
- [ ] 6.9. `respond_to?` reports the methods Class defines natively. `Object.respond_to?(:allocate)` and `Object.respond_to?(:superclass)` answer false in metorex and true in MRI, as does `RubyVM::InstructionSequence.respond_to?(:allocate)`, though each call works. Acceptance: `respond_to?` and `Class.instance_methods(false)` answer what MRI's do for `allocate`, `superclass`, `subclasses`, `attached_object` and `new`
- [ ] 6.10. The debug inspector names the frames a C extension sees as MRI does. The C method running when `rb_debug_inspector_open` is called is located at `"CDebug#frames"` and line 0 in MRI, and at `"frames"` and its caller's line in metorex, as `tests/_examples/c_extensions/c_debug.rb` shows. Frames captured while `require` loads an extension include MRI's `Kernel#require` frame, which metorex leaves out (see 6.8). Acceptance: that example's output matches MRI's with the extension built for MRI
- [ ] 6.11. `Thread#backtrace` of a waiting thread matches MRI's. For a thread blocked in `held.lock` written on line 3, metorex answers `["bt.rb:0:in '<block>'", "bt.rb:0:in '<main>'"]` where MRI answers `["bt.rb:3:in 'Thread::Mutex#lock'", "bt.rb:3:in 'block in <main>'"]`, and for one in `sleep` on line 6 it answers `["bt.rb:6:in 'sleep'", "bt.rb:6:in '<block>'", "bt.rb:0:in '<main>'"]` where MRI answers `["bt.rb:6:in 'Kernel#sleep'", "bt.rb:6:in 'block in <main>'"]`. The deadlock listing reads the same backtraces. Acceptance: those two backtraces match MRI's
- [ ] 6.12. `Process::Status` reports what MRI's does. `p $?` prints `#<Process::Status:0x...>` where MRI prints `#<Process::Status: pid 6895 SIGKILL (signal 9)>`, and `to_s` answers the same where MRI answers `pid 6894 exit 0`. `coredump?` is not defined, and `stopped?` is always false, so `Process.waitpid2(pid, Process::WUNTRACED)` on a stopped child does not answer MRI's `stopped SIGSTOP (signal 17)`. `system("exit 3")` answers 127, since a command that starts with a shell builtin is not run through `/bin/sh` as MRI runs it. Acceptance: `to_s`, `inspect`, `stopped?`, `stopsig` and `coredump?` match MRI for an exited, a signaled and a stopped child, and `system("exit 3")` leaves `$?.exitstatus` at 3
- [ ] 6.13. Addresses come back in the order the operating system answers them. `Socket.resolved` puts IPv4 addresses before IPv6 ones, so on macOS `Addrinfo.tcp("localhost", 80)` is `127.0.0.1:80` where MRI's is `[::1]:80`, and `Addrinfo.getaddrinfo(nil, 80, nil, :STREAM)` lists `127.0.0.1` before `::1` where MRI lists `::1` first. An empty host differs too: `Addrinfo.getaddrinfo("", 80, :INET6, :STREAM)` answers `[]` where MRI raises `Socket::ResolutionError`, and `Addrinfo.getaddrinfo("", 80, :INET, :STREAM)` inspects as `0.0.0.0:80 TCP` where MRI shows `0.0.0.0:80 TCP ()`. Acceptance: those four match MRI on macOS and Linux
- [ ] 6.14. The core methods metorex lacks and Time's ancestors. `Array#rfind` and `Method#box` raise NoMethodError, where MRI 4.0 defines both, and `Time.ancestors` ends `[Comparable, Object]` where MRI's ends `[Kernel, BasicObject]`. Acceptance: `[1, 2, 3].rfind { _1 < 3 }` answers 2, `Method#box` answers what MRI's does, and `Time.ancestors` matches MRI's

## 7. Platforms and guarded specs

ruby/spec wraps code in guards that skip it on some platforms and versions.
The counts are the number of guard calls in `ruby/spec`.

- [ ] 7.1. `platform_is :linux` (42) and `platform_is :darwin` (11) blocks run on the matching machine. Acceptance: `./test.sh` reports the example counts of each platform, and the counts are recorded here for mac, linux/amd64 and linux/arm64
- [ ] 7.2. `as_superuser` (13) blocks run as root in the Linux containers `scripts/linux.sh` starts, with the passing count recorded
- [ ] 7.3. `big_endian` (20) blocks run on a big-endian target (s390x under emulation), or the item records that no big-endian target is available
- [ ] 7.4. `ruby_version_is` (496) blocks are chosen by `RUBY_VERSION`, which metorex reports as 4.0.1. Acceptance: `RUBY_VERSION` and `RUBY_PATCHLEVEL` match the MRI release metorex follows (4.0.6 on this machine), and the specs still pass
- [ ] 7.5. `quarantine!` (22) examples, which never run anywhere, are run once under metorex and their results recorded
- [ ] 7.6. `platform_is :windows` (198) blocks. Metorex runs on Linux and macOS only, so this item records that Windows is out of scope rather than adding support
- [ ] 7.7. The two specs that run only on native machines (`core/process/set_proctitle_spec.rb` and `language/predefined_spec.rb`, skipped when `METOREX_EMULATED=1`) pass on native Linux

## 8. Performance

No spec measures speed, so these items assert on work done, never on time.

- [x] 8.1. Calling a block does not copy its scope. Each call of a block written at the top level copied every name the top level held into a new scope, 388 names in an empty program, because constants lived in the scope the program ran in. `10_000.times { }` took 5.5 s in a debug build and now takes 0.47 s. The program now runs in a scope of its own beneath the one holding the core library, so the program scope holds no constants and a method or class body no longer sees a top-level local. A block's scope reads and writes the names it closed over through one map every call shares, and `Scope::define_captured`, which copied them, is gone
- [ ] 8.2. A benchmark script under `benchmarks/` comparing metorex with MRI on method calls, block calls, string building, hash access and object allocation, run by hand and not by `./test.sh`, with the results recorded in this file
- [ ] 8.3. Method lookup caching: repeated calls of the same method on the same class resolve once until a class or module in its ancestry changes. Acceptance: a test-only lookup counter stays at 1 across 1,000 calls
- [ ] 8.4. A `Hash` lookup that misses with a key that is not an Integer, String, Symbol, Float, nil, true or false sends `eql?` to every key the hash holds, so each miss costs time in proportion to the hash's size. `h = {}; 1200.times { |i| h[[i, -1]] }` with 1,200 array keys stored takes 6.3 seconds in a debug build, against 0.009 seconds for 1,200 hits. Acceptance: such a key is placed by its `#hash` the way an object with its own `#hash` is, the walk over every key is gone, and an example counts the `eql?` calls one miss makes, which is at most the number of keys sharing its hash
