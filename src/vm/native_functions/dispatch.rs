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
            "p" | "pp" => self.inspect_values(arguments, position),
            "readline" => self.read_line_or_fail(arguments, position),
            "readlines" => self.read_all_lines(arguments, position),
            "gets" => self.read_line(arguments, position),
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
