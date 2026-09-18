# Rex, the built-in spec framework

Goal: ship an RSpec clone inside Metorex itself, under the name Rex. A spec file runs with no gem, no `Gemfile`, and no `require`. `metorex spec` is a subcommand of the language, the way `describe` is part of the language.

This work starts after [MSPEC_ROADMAP.md](MSPEC_ROADMAP.md) is complete. That order matters: mspec drives `ruby/spec` through the same metaprogramming surface RSpec needs (`instance_exec`, `define_method`, `method_missing`, singleton classes, `prepend`, `Method#parameters`, `caller`, `at_exit`, `Thread`, `StringIO`, `Dir`, `ENV`). By the time the mspec roadmap closes, those are proven against the official suite, so this roadmap never has to stop and build a language feature first.

## Compatibility target

RSpec 3.13: the three libraries a user sees as one, `rspec-core`, `rspec-expectations`, and `rspec-mocks`. `rspec-rails` is out of scope. `rspec-support` has no public surface and is absorbed.

The acceptance test for the whole effort is a third-party suite written against real RSpec producing the same pass and fail counts and the same documentation-format output under `metorex spec`.

## The name

The framework is `Rex`, from the tail of METOREX (Meta Object Runtime EXecution). `RSpec` is a constant alias for the same module object, defined at load, so `RSpec.describe`, `RSpec::Matchers`, and `RSpec::Expectations::ExpectationNotMetError` all resolve and an existing suite runs unchanged. One implementation, two names.

```ruby
Rex.describe Stack do
  let(:stack) { Stack.new }

  it "pushes" do
    stack.push 1
    expect(stack.size).to eq 1
  end
end

Rex.configure do |config|
  config.order = :random
end
```

The internal split mirrors the three gems, so the alias lands on identical paths:

- `Rex::Core` for `ExampleGroup`, `Example`, `World`, `Configuration`, and the formatters
- `Rex::Expectations` for `expect`, the matcher protocol, and the failure class
- `Rex::Mocks` for doubles and message expectations
- `Rex::Matchers` for the matcher library and `Matchers.define`

Where RSpec's own name is longer than it needs to be, the Rex path is the short one and RSpec's is the alias. `Rex::ExpectationNotMet` is the class, and `Rex::Expectations::ExpectationNotMetError` points at it.

`require "rspec"` and `require "rspec/autorun"` succeed and load nothing. File conventions stay RSpec's, so `spec/`, `*_spec.rb`, and `spec/spec_helper.rb` need no moves in an existing project, and the runner reads both `.rspec` and `.rex` for options.

A `rex` gem exists (Metasploit's Ruby Exploitation library), as does a `rex` lexer generator. Metorex loads no gems, so neither can claim the constant.

## Correct first, then fast

The clone is built correct and then made fast, in that order. Every optimization needs a test that catches a behavior change, and that test is rspec-core's own suite running green, which is the last item of the compatibility proof. Building the fast path first leaves nothing to say when it lies.

Speed comes from a native fast path guarded by a redefinition check, deoptimizing to the Ruby definition the moment a suite changes anything observable. This is what MRI does for `Integer#+`. Compatibility is defined by behavior through the public API, not by the language a method is written in, so `Rex::Matchers#eq` stays a real Ruby method either way and the native path is an optimization of calling it.

The rule the speed work is held to: if a suite can tell the difference by any means other than a clock, the optimization is wrong.

Parallel execution is the exception to the ordering. It changes no observable behavior of a single example, so it stays where it is in the plan rather than waiting for the proof.

## Where the code lives

Three layers, chosen per feature by what the layer above cannot do:

- **`src/vm/prelude.rs`** holds the DSL, the matchers, and the reporters, written in Ruby. A matcher is then a real user-defined class, so a spec can subclass it, alias it, or redefine it, and `Rex::Matchers` opens the way a user's own module does.
- **Rust native primitives** cover what Ruby in the prelude cannot reach: the stub and restore transaction on the method table, source extraction for an expression's AST, the LCS diff, monotonic timing, the seeded RNG, terminal width and color detection, and the example-status persistence file.
- **`src/main.rs` and a new `src/spec/`** hold the runner: argument parsing, `.rspec` files, file and line filtering, load ordering, and the exit code.

## What a built-in runner does that a gem cannot

- **Source-aware failure messages.** Metorex exposes the AST as runtime objects, so `expect(user.age).to eq(42)` can report `expected user.age to eq 42, got 41` with the real expression text rather than `expected 42, got 41`.
- **Stubbing as a runtime transaction.** `allow(obj).to receive(:foo)` swaps a method table entry and records an undo record in the VM. Restoration is exact, it cannot leak across examples, and it needs no `alias_method` shuffling or `method_missing` shim.
- **Verifying doubles with real signatures.** `instance_double` reads `Method#parameters` from the VM, so a keyword argument typo fails when the double is set up.
- **Backtrace filtering by frame ownership.** A framework frame is marked when it is pushed, so filtering is exact rather than a regex over file paths.
- **`let` resolution in a real scope.** A per-example scope in the VM holds the memoized values, so a misspelled `let` name raises a NameError naming the helpers declared near it.
- **Coverage with no second tool.** The VM already walks lines, so `metorex spec --coverage` reports line and branch coverage with no instrumentation pass.

## 1. Runner skeleton and the CLI

- [ ] 1.1. `metorex spec [files-or-directories]` subcommand, defaulting to `spec/`
- [ ] 1.2. File discovery: `**/*_spec.rb` under the given paths, sorted for a deterministic default load order
- [ ] 1.3. `spec/spec_helper.rb` loaded when present, before any spec file
- [ ] 1.4. `.rspec` and `.rspec-local` option files, plus the `SPEC_OPTS` environment variable, merged in RSpec's precedence order
- [ ] 1.5. `--require`, `--default-path`, `--pattern`, `--exclude-pattern`
- [ ] 1.6. Location filters: `path/to/file_spec.rb:42`, repeatable, and `--example` for a name substring
- [ ] 1.7. Exit status: 0 for a clean run, 1 for any failure, and `--failure-exit-code`
- [ ] 1.8. `Rex::Core::World` singleton holding registered groups, filters, and the reporter
- [ ] 1.9. Examples under `tests/_examples/rex/runner/` wired into `examples_runner.rs`

## 2. Example group DSL

- [ ] 2.1. `Rex.describe` and top-level `describe`, with a String, a Class, or a Module as the subject argument
- [ ] 2.2. Group aliases: `context`, `xdescribe`, `xcontext`, `fdescribe`, `fcontext`, and `Rex.context`
- [ ] 2.3. Nested groups to any depth, each a subclass of its parent group class
- [ ] 2.4. `it`, `specify`, `example`, and their `x` and `f` forms
- [ ] 2.5. Description inference for `it` with no description, read from the matcher
- [ ] 2.6. `pending` and `skip`, as a method inside an example and as an argument on the group or the example
- [ ] 2.7. A pending example that passes reported as a failure, with RSpec's wording
- [ ] 2.8. `described_class` and the full `description` string built by walking the group chain
- [ ] 2.9. `Rex::Core::ExampleGroup` as a real class, so `self` inside an example is an instance of it
- [ ] 2.10. Examples under `tests/_examples/rex/dsl/`

## 3. Hooks, helpers, and memoized values

- [ ] 3.1. `before` and `after` with `:each` and `:example`, running outside-in and inside-out
- [ ] 3.2. `before` and `after` with `:all` and `:context`
- [ ] 3.3. `before(:suite)` and `after(:suite)` through `Rex.configure`
- [ ] 3.4. `around(:each)` with a yielded `Example` answering `run` and `call`
- [ ] 3.5. `let` and `let!`, memoized per example, redefinable in a nested group
- [ ] 3.6. `subject` and named `subject(:name)`, plus the implicit subject built from the described class
- [ ] 3.7. `is_expected` and `should` on the implicit subject, for one-liner examples
- [ ] 3.8. Helper modules through `config.include`, `config.extend`, and `config.prepend`, each with metadata filtering
- [ ] 3.9. A NameError for an undefined `let` that names the helpers declared nearby
- [ ] 3.10. Hook ordering across a group chain proved by an example that appends to an array
- [ ] 3.11. Examples under `tests/_examples/rex/hooks/`

## 4. The expectation protocol

- [ ] 4.1. `expect(value)` and `expect { }` answering an `ExpectationTarget`
- [ ] 4.2. `to`, `not_to`, and `to_not`
- [ ] 4.3. The matcher protocol: `matches?`, `does_not_match?`, `failure_message`, `failure_message_when_negated`, `description`, `supports_block_expectations?`
- [ ] 4.4. `Rex::ExpectationNotMet` raised on failure, carrying the message and the caller location, with `Rex::Expectations::ExpectationNotMetError` aliased to it
- [ ] 4.5. `expect` reachable inside an example with no receiver, and inside a custom matcher block
- [ ] 4.6. `Rex::Matchers.define` for custom matchers, with `match`, `match_when_negated`, `failure_message`, `description`, and the `expected` and `actual` readers
- [ ] 4.7. Matcher DSL argument access: `expected`, `actual`, `expected_as_notation`
- [ ] 4.8. The `should` and `should_not` syntax on every object, off by default and turned on by `config.expect_with`
- [ ] 4.9. `config.expect_with(:rspec) { |c| c.syntax = ... }` honored by both syntaxes
- [ ] 4.10. Examples under `tests/_examples/rex/expectations/`

## 5. Built-in matchers

- [ ] 5.1. Equality: `eq`, `eql`, `equal`, `be`
- [ ] 5.2. Comparison: `be >`, `be >=`, `be <`, `be <=`, `be_between(...).inclusive`, `be_within(...).of(...)`
- [ ] 5.3. Type and identity: `be_a`, `be_an`, `be_kind_of`, `be_instance_of`, `respond_to(...).with(n).arguments`, `.with_keywords`
- [ ] 5.4. Truthiness: `be_truthy`, `be_falsey`, `be_nil`
- [ ] 5.5. Predicate matchers through `method_missing`: `be_empty`, `be_valid`, `have_key`, and the question-mark fallback
- [ ] 5.6. Collections: `include`, `contain_exactly`, `match_array`, `start_with`, `end_with`, `all`
- [ ] 5.7. `match` against a Regexp, a String, or a structure of nested matchers
- [ ] 5.8. Errors: `raise_error` with a class, a message String, a Regexp, a block, and the bare form with its warning
- [ ] 5.9. `throw_symbol` and `have_attributes`
- [ ] 5.10. `change { }` with `by`, `from`, `to`, `by_at_least`, `by_at_most`
- [ ] 5.11. `output(...).to_stdout`, `.to_stderr`, and the `_from_any_process` forms
- [ ] 5.12. `yield_control`, `yield_with_args`, `yield_with_no_args`, `yield_successive_args`
- [ ] 5.13. `satisfy`, the operator matchers, and the compound combinators `and` and `or`
- [ ] 5.14. `aggregate_failures`, as a block method and as example metadata, reporting every failure in one message
- [ ] 5.15. Matcher composition: any matcher accepted where a value is accepted, as in `include(a_string_matching(/x/))`
- [ ] 5.16. Aliased matchers: `a_string_matching`, `an_instance_of`, and the rest of RSpec's alias table
- [ ] 5.17. Examples under `tests/_examples/rex/matchers/`, one file per matcher family

## 6. Doubles and message expectations

- [ ] 6.1. A VM stub transaction: install a method table entry, record the undo, restore when the example ends
- [ ] 6.2. `double`, `double("name")`, `double(name: value)`, and the strict-double error for an unstubbed message
- [ ] 6.3. `allow(obj).to receive(:msg)`, with `and_return`, `and_raise`, `and_throw`, `and_yield`, `and_call_original`
- [ ] 6.4. Several return values from one `and_return`, consumed in order
- [ ] 6.5. `expect(obj).to receive(:msg)`, verified when the example ends, with the unfulfilled-expectation message
- [ ] 6.6. Receive counts: `once`, `twice`, `exactly(n).times`, `at_least`, `at_most`, `never`
- [ ] 6.7. Argument constraints: `with(...)`, `anything`, `any_args`, `no_args`, `hash_including`, `array_including`, `instance_of`, `kind_of`, and any matcher
- [ ] 6.8. `receive_messages` and `receive_message_chain`
- [ ] 6.9. Spies: `spy`, `have_received`, and `allow(obj).to receive(:msg)` followed by `expect(obj).to have_received(:msg)`
- [ ] 6.10. Partial doubles on a real object, with the original method restored exactly
- [ ] 6.11. Verifying doubles: `instance_double`, `class_double`, `object_double`, checking existence, arity, and keywords through `Method#parameters`
- [ ] 6.12. `verify_partial_doubles` configuration, on by default
- [ ] 6.13. `allow_any_instance_of` and `expect_any_instance_of`
- [ ] 6.14. `config.mock_with` and the `should_receive` syntax behind it
- [ ] 6.15. A leakage spec: a file whose second example asserts the first example's stub is gone
- [ ] 6.16. Examples under `tests/_examples/rex/mocks/`

## 7. Metadata, filtering, and shared code

- [ ] 7.1. Metadata on groups and examples, as a Hash and as bare Symbols, inherited by nested groups
- [ ] 7.2. Built-in metadata keys: `described_class`, `full_description`, `file_path`, `line_number`, `location`, `scoped_id`
- [ ] 7.3. `config.filter_run_when_matching :focus`, with `fit` and `fdescribe` applying it
- [ ] 7.4. `--tag` and `--tag ~name`, `config.filter_run_including`, `config.filter_run_excluding`
- [ ] 7.5. Conditional filters, where a filter value is a Proc taking the metadata
- [ ] 7.6. `shared_examples`, `shared_examples_for`, `shared_context`, each with a name, a Symbol, or a Module
- [ ] 7.7. `it_behaves_like`, `include_examples`, `it_should_behave_like`, with block arguments and a customization block
- [ ] 7.8. `config.include_context` with metadata, and `config.shared_context_metadata_behavior`
- [ ] 7.9. Examples under `tests/_examples/rex/metadata/`

## 8. Ordering, seeds, and run persistence

- [ ] 8.1. `--order defined`, `--order random`, `--order rand:SEED`, and `config.order`
- [ ] 8.2. A seeded RNG native primitive, so one seed reproduces one ordering
- [ ] 8.3. `config.register_ordering` for a named custom ordering, plus `:global`
- [ ] 8.4. The example status persistence file, `config.example_status_persistence_file_path`
- [ ] 8.5. `--only-failures` and `--next-failure`, reading that file
- [ ] 8.6. `--fail-fast` and `--fail-fast=N`
- [ ] 8.7. `--dry-run`, which builds every group and runs no example body
- [ ] 8.8. `--bisect`, narrowing an ordering-dependent failure to the smallest reproducing set
- [ ] 8.9. Examples under `tests/_examples/rex/ordering/`

## 9. Reporting and formatters

- [ ] 9.1. The formatter protocol and the notification objects RSpec hands it
- [ ] 9.2. `progress` formatter, the default: a dot per example, `F` for a failure, `*` for pending
- [ ] 9.3. `documentation` formatter, indented by group nesting
- [ ] 9.4. The failure block: description, message, the failing source line, and the filtered backtrace
- [ ] 9.5. Color detection from the terminal, `--color`, `--no-color`, `--force-color`
- [ ] 9.6. The summary line, the failed-examples rerun list, and the seed line
- [ ] 9.7. `json` formatter, matching RSpec's document shape
- [ ] 9.8. `html` formatter
- [ ] 9.9. `--profile [N]`, ranked by monotonic example duration
- [ ] 9.10. `--format` and `--out`, repeatable, so one run writes several formats
- [ ] 9.11. `config.add_formatter` and a user-defined formatter class
- [ ] 9.12. Output captured to a StringIO and asserted in `tests/_examples/rex/formatters/`

## 10. Diffs and source-aware messages

- [ ] 10.1. An LCS diff native primitive, so no `diff-lcs` gem is involved
- [ ] 10.2. Unified diff output for two Strings that differ, colorized
- [ ] 10.3. Object diffs through `pretty_print`, for Hashes, Arrays, and Structs
- [ ] 10.4. `config.diff_enabled`, `--no-diff`, and the size threshold that suppresses a diff
- [ ] 10.5. Source text of the expression handed to `expect`, read from the AST the VM already holds
- [ ] 10.6. Failure messages that name the expression: `expected user.age to eq 42, got 41`
- [ ] 10.7. `config.source_expressions`, so a suite can turn the expression text off
- [ ] 10.8. The failing line printed with the lines around it
- [ ] 10.9. Examples under `tests/_examples/rex/diffs/`

## 11. Backtraces and error handling

- [ ] 11.1. Frame ownership marked when a framework frame is pushed, rather than matched by path
- [ ] 11.2. `config.backtrace_exclusion_patterns` and `config.backtrace_inclusion_patterns` layered on top
- [ ] 11.3. `--backtrace` for the unfiltered form
- [ ] 11.4. An error raised in `before(:all)` failing every example in the group, with RSpec's wording
- [ ] 11.5. An error raised in `after` reported beside the example's own result
- [ ] 11.6. An error outside any example reported as `An error occurred while loading <file>`, with the run continuing
- [ ] 11.7. `config.raise_errors_for_deprecations!` and the deprecation stream
- [ ] 11.8. Interrupt handling: a signal ends the run and prints the summary for what ran
- [ ] 11.9. Examples under `tests/_examples/rex/errors/`

## 12. Configuration surface

- [ ] 12.1. `Rex.configure` and the `Configuration` object, with every setting reachable as a reader and a writer
- [ ] 12.2. `config.disable_monkey_patching!`
- [ ] 12.3. `config.expose_dsl_globally` and the main-object DSL
- [ ] 12.4. `config.define_derived_metadata`
- [ ] 12.5. `config.when_first_matching_example_defined`
- [ ] 12.6. `config.around`, `config.before`, `config.after` with a scope and metadata
- [ ] 12.7. `config.default_formatter`, `config.silence_filter_announcements`, `config.warnings`
- [ ] 12.8. `Rex::Core::Pending` and the remaining legacy settings an existing `spec_helper.rb` may set
- [ ] 12.9. `RSpec` aliased to `Rex` at load, and `config.rspec_constant` to leave it undefined
- [ ] 12.10. Examples under `tests/_examples/rex/config/`

## 13. Parallel execution

- [ ] 13.1. Group-level work distribution across Threads, with a per-worker reporter
- [ ] 13.2. `--jobs N`, defaulting to the machine's core count
- [ ] 13.3. Output serialized per group, so a parallel documentation run reads the same as a serial one
- [ ] 13.4. Shared-state detection: a warning when a parallel run and a serial run disagree under one seed
- [ ] 13.5. `--jobs 1` as the exact equivalent of a serial run, proved by identical output bytes
- [ ] 13.6. Examples under `tests/_examples/rex/parallel/`

## 14. Coverage

- [ ] 14.1. `metorex spec --coverage`, reading the line table the VM already walks
- [ ] 14.2. Branch coverage from the bytecode's jump targets
- [ ] 14.3. `--coverage-format` for a summary, an LCOV file, and a JSON file
- [ ] 14.4. `--coverage-threshold N`, failing the run below it
- [ ] 14.5. Filters for which files count, defaulting to everything loaded that is not a spec file
- [ ] 14.6. Examples under `tests/_examples/rex/coverage/`

## 15. Compatibility proof

- [ ] 15.1. Port the `tests/_examples/` assertions that already read as specs into `spec/` files run by `metorex spec`
- [ ] 15.2. Run `rspec-expectations`' own spec suite unchanged, and record the pass count
- [ ] 15.3. Run `rspec-mocks`' own spec suite unchanged, and record the pass count
- [ ] 15.4. Run `rspec-core`'s own spec suite unchanged, and record the pass count
- [ ] 15.5. Pick a third-party library with an RSpec suite and no C extensions, run it, and record the result
- [ ] 15.6. Byte-compare `documentation` formatter output against real RSpec for a fixture suite
- [ ] 15.7. A `scripts/run_rspec_suite.sh` driver shaped like `scripts/run_ruby_spec.sh`, with a commented list advanced one line at a time
- [ ] 15.8. README section describing the built-in runner, its flags, and what it does that the gem cannot

## 16. Speed

Begins after the compatibility proof passes. Each item lands with its guard and with a spec that redefines the thing it optimizes and proves the deopt fires.

- [ ] 16.1. A benchmark fixture suite of a few thousand trivial examples, plus a recorded baseline checked into the repo
- [ ] 16.2. `scripts/bench_rex.sh`, reporting boot time, examples per second, and peak memory against that baseline
- [ ] 16.3. CI runs the compatibility suites twice, fast paths on and off, and byte-compares the output
- [ ] 16.4. `--no-fast-paths` and `config.fast_paths`, so any divergence can be bisected by a user
- [ ] 16.5. A method-table redefinition counter the fast paths read, shared with the VM's inline caches
- [ ] 16.6. Prelude snapshot, or a split prelude whose spec half loads on `metorex spec` and on the first `describe`, whichever measures better
- [ ] 16.7. Hook chains flattened into one ordered list when the group is defined, rather than walked per example
- [ ] 16.8. `let` compiled to a method plus a memo slot in the example scope, with `__memoized` kept as a compatible view
- [ ] 16.9. Metadata populated natively, still a mutable Hash the suite can write to
- [ ] 16.10. Example group instances allocated natively, deoptimizing for a group that defines `initialize`
- [ ] 16.11. Native `eq`, `be`, `be_truthy`, `be_nil`, `include`, and `match`, each behind its guard
- [ ] 16.12. Native double teardown, so a mock-heavy suite pays the undo record rather than a global space walk
- [ ] 16.13. Profile one third-party suite before and after, and record both numbers in the README
- [ ] 16.14. Examples under `tests/_examples/rex/performance/`, one per guard, each redefining the optimized method and asserting the Ruby path runs

## Open questions

- **`--bisect` transport.** Real RSpec shells out to a second process per bisection round. Metorex can run a round in-process on a fresh VM, which is faster and cannot inherit environment drift. Decide while building the ordering work.
- **Monkey patching.** RSpec's top-level `describe` is a method on `main`. Since this ships in the language, `disable_monkey_patching!` could be the default and `Rex.describe` the only form. That breaks existing suites, so the plan keeps RSpec's default and leaves the setting to the user.
- **Load cost.** Building the whole framework into every VM costs startup for a script that never runs a spec. Either the prelude splits and the spec half loads on `metorex spec` and on the first `describe`, or the prelude is snapshotted after compilation. Measure before choosing.
