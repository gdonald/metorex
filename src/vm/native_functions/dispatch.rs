// Which function a name stands for. Every arm hands the call on to a method
// of its own, grouped into the modules beside this one.

use super::*;

impl VirtualMachine {
    /// Call a native function by name.
    pub(crate) fn call_native_function(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let entered = self.enter_native_frame(None, name, position);
        let answered = self.call_native_function_body(name, arguments, position);
        self.leave_native_call(entered, position, &answered);
        answered
    }

    fn call_native_function_body(
        &mut self,
        name: &str,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // The receiver the call was written with reaches the function invoked
        // here and no further, so a function that runs code of its own does
        // not hand it on to whatever that code calls.
        let invoked_with = self.kernel_function_receiver.take();
        match name {
            "private" | "public" | "protected" => self.apply_visibility(name, arguments, position),
            "private_constant" | "public_constant" | "deprecate_constant" => {
                self.apply_constant_visibility(name, arguments)
            }
            "module_function" => self.apply_module_function(arguments, position),
            "private_class_method" | "public_class_method" => {
                self.apply_class_method_visibility(name, arguments, position)
            }
            "noop_with_block" => {
                self.pending_block.take();
                Ok(Object::Nil)
            }
            "freeze" => self.freeze_value(),
            "lambda" => self.make_lambda(position),
            "proc" => self.make_proc(position),
            "__end_once__" => self.end_once(position),
            "at_exit" => self.register_at_exit(position),
            "using" => self.use_refinement(arguments, position),
            "warn" => self.kernel_warn(arguments, position),
            // `trap` is Kernel's name for `Signal.trap`.
            "trap" => self.install_signal_trap(&arguments, position),
            "sprintf" => self.format_string(arguments, position),
            "__method__" | "__callee__" => self.running_method_name(name),
            "caller" => self.caller_backtrace(arguments, position),
            "caller_locations" => self.caller_location_list(arguments, position),
            "binding_kernel" => self.kernel_binding(position),
            "top_level_to_s" => Ok(Object::string("main".to_string())),
            "define_method" => self.define_top_level_method(arguments, position),
            "rand" => self.random_draw(arguments, position),
            "trace_var" => self.trace_global(arguments, position),
            "untrace_var" => self.untrace_global(arguments, position),
            "srand" => self.seed_generator(arguments, position),
            "sleep" => self.sleep_for(arguments, position),
            "__timeout_open__" => self.open_timeout(arguments, position),
            "__continuation_site__" => Ok(self.continuation_site()),
            "__continuation_resumed__" => Ok(self.continuation_resumed(&arguments)),
            "__continuation_resume__" => self.continuation_resume(&arguments, position),
            "__singleton_given__" => Ok(Object::Bool(
                arguments
                    .first()
                    .is_some_and(|given| self.singleton_given(given)),
            )),
            "iterator?" => Ok(self.iterator_query(position)),
            // Ruby leaves the choice of instance variables to the default
            // `inspect` unless a class says otherwise.
            "instance_variables_to_inspect" => Ok(Object::Nil),
            // The names `require` finds a library metorex carries under,
            // which did_you_mean suggests from alongside the load path.
            // The source of a program given on the command line or on
            // standard input, which has no file to read it back from.
            "__console_mode_get__"
            | "__console_mode_set__"
            | "__console_mode_change__"
            | "__console_mode_query__"
            | "__console_winsize__"
            | "__console_set_winsize__"
            | "__console_flush__"
            | "__console_beep__"
            | "__console_ttyname__" => self.call_console_function(name, &arguments),
            "__prism_version__"
            | "__prism_serialize__"
            | "__prism_serialize_stream__"
            | "__prism_parse_success__"
            | "__prism_string_query__" => self.call_prism_function(name, &arguments, position),
            "__embedded_library_names__" => Ok(Object::array(
                crate::vm::stdlib::embedded_library_names()
                    .map(|name| Object::string(name.to_string()))
                    .collect(),
            )),
            // A class whose instances cannot be made, as C's
            // `rb_undef_alloc_func` says of one.
            "__undefine_allocator__" => {
                if let Some(Object::Class(class)) = arguments.first() {
                    class.set_class_var(crate::vm::capi::data_allocator_var(), Object::Int(0));
                }
                Ok(Object::Nil)
            }
            // An instance of a class whose allocator is undefined, which the
            // core library makes for itself.
            "__allocate_instance__" => match arguments.first() {
                Some(Object::Class(class)) => self.allocate_instance(&Rc::clone(class), position),
                _ => Ok(Object::Nil),
            },
            // The converter behind RubyVM::AbstractSyntaxTree, read the
            // first time a program asks for a syntax tree.
            "__load_abstract_syntax_tree__" => {
                self.run_embedded_library(
                    "abstract_syntax_tree",
                    include_str!("../stdlib/abstract_syntax_tree.rb"),
                )?;
                Ok(Object::Nil)
            }
            // The converter behind RubyVM::InstructionSequence, read the first
            // time a program compiles one.
            "__load_instruction_sequence__" => {
                self.run_embedded_library(
                    "instruction_sequence",
                    include_str!("../stdlib/instruction_sequence.rb"),
                )?;
                Ok(Object::Nil)
            }
            // Where a block literal opens, as its line and its column counted
            // in characters from 0, or nil for a block with no Ruby source.
            "__block_position__" => Ok(match arguments.first() {
                Some(Object::Block(block)) => match (block.opened_at, block.opened_column) {
                    (Some(line), Some(column)) => {
                        Object::array(vec![Object::Int(line as i64), Object::Int(column as i64)])
                    }
                    _ => Object::Nil,
                },
                _ => Object::Nil,
            }),
            "__ractor_move__" => {
                if let Some(moved) = arguments.into_iter().next() {
                    self.mark_moved(moved);
                }
                Ok(Object::Nil)
            }
            "__timeout_close__" => {
                self.timeout_limits.pop();
                Ok(Object::Nil)
            }
            // The primitive behind the Math module: the function named by the
            // first argument, applied to the numbers that follow.
            "__math_function__" => self.apply_math_function(&arguments, position),
            // The binary running this program, which is what `RbConfig.ruby`
            // names for starting another one.
            "__interpreter_path__" => Ok(match std::env::current_exe() {
                Ok(path) => Object::string(path.display().to_string()),
                Err(_) => Object::Nil,
            }),
            // Where the headers a C extension is compiled against live.
            "__header_directory__" => Ok(Object::string(crate::vm::capi::HEADER_DIRECTORY)),
            // The bytes a C type's size function counts for what an object
            // wraps, which `ObjectSpace.memsize_of` adds.
            "__wrapped_size__" => Ok(Object::Int(
                arguments.first().map_or(0, crate::vm::capi::wrapped_size) as i64,
            )),
            "puts" => self.put_lines(arguments, position),
            "method" => self.method_object(arguments, position),
            "autoload" | "autoload?" => self.register_autoload(name, arguments, position),
            "`" => self.shell_command(arguments, position),
            "chomp" | "chop" => self.chomp_line(name, arguments),
            "open" => self.open_stream(arguments, position),
            "require" => self.require_feature(arguments, position),
            "__resolve_feature_path__" => self.resolve_feature_path(arguments, position),
            // A handle naming an object without keeping it alive, and the
            // object a handle names while anything else still holds it.
            "__weak_reference__" => {
                let target = arguments.first().cloned().unwrap_or(Object::Nil);
                self.weak_references
                    .push(crate::vm::core::WeakTarget::of(&target));
                Ok(Object::Int(self.weak_references.len() as i64 - 1))
            }
            "__weak_target__" => Ok(match arguments.first() {
                Some(Object::Int(handle)) => usize::try_from(*handle)
                    .ok()
                    .and_then(|at| self.weak_references.get(at))
                    .and_then(|target| target.reach())
                    .unwrap_or(Object::Nil),
                _ => Object::Nil,
            }),
            "require_relative" => self.require_relative_feature(arguments, position),
            "print" => self.print_values(arguments, position),
            "printf" => self.print_formatted(arguments, position),
            "p" => self.inspect_values(arguments, position),
            "pp" => self.pretty_print_values(arguments, position),
            "readline" => self.read_through_argf("readline", arguments, position),
            "readlines" => self.read_through_argf("readlines", arguments, position),
            "gets" => self.read_through_argf("gets", arguments, position),
            "assert" => self.assert_true(arguments, position),
            "assert_equal" => self.assert_equality(arguments, position),
            "assert_raises" => self.assert_raises_exception(arguments, position),
            "parse" => self.parse_source(arguments, position),
            "eval" => self.eval_source(arguments, position, invoked_with),
            "global_variables" => self.global_variable_names(arguments, position),
            "local_variables" => self.local_variable_names(arguments, position),
            "fail" | "raise" => self.raise_exception(arguments, position),
            "loop" => self.run_loop(arguments, position),
            "catch" => self.catch_tag(arguments, position),
            "throw" => self.throw_tag(arguments, position),
            "load" => self.load_file(arguments, position),
            "exit" | "exit!" => self.exit_program(name, arguments, position),
            "abort" => self.abort_program(arguments, position),
            "exec" => self.exec_program(arguments, position),
            "spawn" => self.spawn_program(arguments, position),
            "system" => self.system_command(arguments, position),
            "fork" => self.fork_process(position),
            _ => Err(MetorexError::runtime_error(
                format!("Unknown native function: {}", name),
                crate::vm::utils::position_to_location(position),
            )),
        }
    }
}
