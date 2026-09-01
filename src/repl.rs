// Metorex REPL
// Interactive Read-Eval-Print Loop for Metorex

use crate::error::MetorexError;
use crate::lexer::Lexer;
use crate::object::Object;
use crate::parser::Parser;
use crate::vm::VirtualMachine;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};

const PROMPT: &str = ">> ";
const CONTINUATION_PROMPT: &str = ".. ";
const BANNER: &str = include_str!("banner.txt");

/// Result of processing a single line of REPL input.
#[derive(Debug, PartialEq)]
pub enum LineResult {
    /// Continue the REPL (nothing special happened).
    Continue,
    /// The user asked to exit.
    Exit,
    /// A value was produced and should be displayed.
    Value(String),
    /// An error occurred and should be displayed.
    Error(String),
    /// Input is incomplete — prompt for continuation.
    Incomplete,
}

/// Command result from handle_command.
#[derive(Debug, PartialEq)]
pub enum CommandResult {
    /// The command was handled; exit the REPL.
    Exit,
    /// The command was handled; continue.
    Continue,
    /// The command reset the VM.
    Reset,
    /// The command cleared the screen; here is the text to print.
    Clear(String),
    /// Help text to display.
    Help(String),
    /// Unknown command.
    Unknown(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// ReplCore — all testable logic, no I/O dependencies
// ─────────────────────────────────────────────────────────────────────────────

pub struct ReplCore {
    vm: VirtualMachine,
    buffer: String,
}

impl ReplCore {
    pub fn new() -> Self {
        Self {
            vm: VirtualMachine::new(),
            buffer: String::new(),
        }
    }

    /// Access the underlying VM (for test inspection).
    pub fn vm(&self) -> &VirtualMachine {
        &self.vm
    }

    /// Current buffer contents.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Whether we are in continuation mode (buffer is non-empty).
    pub fn is_continuation(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// Return the appropriate prompt string.
    pub fn prompt(&self) -> &'static str {
        if self.buffer.is_empty() {
            PROMPT
        } else {
            CONTINUATION_PROMPT
        }
    }

    /// Process one line of input. Returns what the caller should do.
    pub fn process_line(&mut self, line: &str) -> LineResult {
        // Handle dot-commands only when buffer is empty
        if self.buffer.is_empty() && line.trim().starts_with('.') {
            return match self.handle_command(line) {
                CommandResult::Exit => LineResult::Exit,
                CommandResult::Continue => LineResult::Continue,
                CommandResult::Reset => LineResult::Continue,
                CommandResult::Clear(text) => LineResult::Value(text),
                CommandResult::Help(text) => LineResult::Value(text),
                CommandResult::Unknown(msg) => LineResult::Error(msg),
            };
        }

        // Append line to buffer
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(line);

        // Try to evaluate
        if self.should_evaluate() {
            let result = self.evaluate_buffer();
            self.buffer.clear();
            result
        } else {
            LineResult::Incomplete
        }
    }

    /// Clear the input buffer (e.g. on Ctrl-C).
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Handle special REPL commands (lines starting with `.`).
    pub fn handle_command(&mut self, line: &str) -> CommandResult {
        let cmd = line.trim();

        match cmd {
            ".exit" | ".quit" => CommandResult::Exit,
            ".help" => CommandResult::Help(Self::help_text()),
            ".clear" => {
                let text = format!(
                    "{}\nMetorex REPL v{}\nType .help for more information, .exit to quit\n",
                    BANNER,
                    env!("CARGO_PKG_VERSION")
                );
                CommandResult::Clear(text)
            }
            ".reset" => {
                self.vm = VirtualMachine::new();
                CommandResult::Reset
            }
            _ => CommandResult::Unknown(format!(
                "Unknown command: {}\nType .help for available commands",
                cmd
            )),
        }
    }

    /// Generate help text.
    pub fn help_text() -> String {
        let mut s = String::new();
        s.push_str("Metorex REPL Commands:\n");
        s.push_str("  .help       Show this help message\n");
        s.push_str("  .exit       Exit the REPL (or Ctrl-D)\n");
        s.push_str("  .quit       Alias for .exit\n");
        s.push_str("  .clear      Clear the screen\n");
        s.push_str("  .reset      Reset the VM state\n");
        s.push('\n');
        s.push_str("Keyboard shortcuts:\n");
        s.push_str("  Ctrl-C      Clear current input buffer\n");
        s.push_str("  Ctrl-D      Exit the REPL\n");
        s.push('\n');
        s.push_str("Multi-line input:\n");
        s.push_str("  The REPL automatically detects incomplete expressions\n");
        s.push_str("  and prompts for continuation with '..'");
        s
    }

    /// Banner text shown at startup.
    pub fn banner() -> String {
        format!(
            "{}\nMetorex REPL v{}\nType .help for more information, .exit to quit\n",
            BANNER,
            env!("CARGO_PKG_VERSION")
        )
    }

    /// Determine if the current buffer should be evaluated.
    pub fn should_evaluate(&self) -> bool {
        let trimmed = self.buffer.trim();

        if trimmed.is_empty() {
            return true;
        }

        let lexer = Lexer::new(&self.buffer);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        match parser.parse() {
            Ok(_) => true,
            Err(errors) => {
                for error in &errors {
                    let error_msg = error.to_string().to_lowercase();
                    if error_msg.contains("unexpected end of input")
                        || error_msg.contains("expected 'end'")
                        || error_msg.contains("unclosed")
                        || error_msg.contains("incomplete")
                    {
                        return false;
                    }
                }
                true
            }
        }
    }

    /// Evaluate the current buffer. Returns the result for display.
    pub fn evaluate_buffer(&mut self) -> LineResult {
        let lexer = Lexer::new(&self.buffer);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let program = match parser.parse() {
            Ok(prog) => prog,
            Err(errors) => {
                let msgs: Vec<String> = errors
                    .iter()
                    .map(|e| format!("Parse error: {}", e))
                    .collect();
                return LineResult::Error(msgs.join("\n"));
            }
        };

        match self.vm.execute_program(&program) {
            Ok(Some(result)) => {
                if matches!(result, Object::Nil) {
                    LineResult::Continue
                } else {
                    LineResult::Value(format!("=> {}", Self::format_object(&result)))
                }
            }
            Ok(None) => LineResult::Continue,
            Err(err) => LineResult::Error(format!("Runtime error: {}", Self::format_error(&err))),
        }
    }

    /// Format an object for display.
    pub fn format_object(obj: &Object) -> String {
        match obj {
            Object::Nil => "nil".to_string(),
            Object::Bool(b) => b.to_string(),
            Object::Int(i) => i.to_string(),
            Object::BigInt(i) => i.to_string(),
            Object::Float(f) => {
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            }
            Object::String(s) => format!("\"{}\"", s),
            Object::Symbol(s) => format!(":{}", s),
            Object::Array(items) => {
                let items_borrowed = items.borrow();
                let formatted_items: Vec<String> =
                    items_borrowed.iter().map(Self::format_object).collect();
                format!("[{}]", formatted_items.join(", "))
            }
            Object::Dict(map) => {
                let map_borrowed = map.borrow();
                let mut entries: Vec<String> = map_borrowed
                    .iter()
                    .map(|(k, v)| format!("\"{}\" => {}", k, Self::format_object(v)))
                    .collect();
                entries.sort();
                format!("{{{}}}", entries.join(", "))
            }
            Object::Block { .. } => "<Block>".to_string(),
            Object::Method { .. } => "<Method>".to_string(),
            Object::NativeFunction(name) => format!("<NativeFunction: {}>", name),
            Object::Class(class) => format!("<Class: {}>", class.name()),
            Object::Module(module) => format!("<Module: {}>", module.name()),
            Object::Instance(instance) => {
                let instance_borrowed = instance.borrow();
                format!("<{} instance>", instance_borrowed.class_name())
            }
            Object::Range {
                start,
                end,
                exclusive,
            } => {
                if *exclusive {
                    format!(
                        "{}...{}",
                        Self::format_object(start),
                        Self::format_object(end)
                    )
                } else {
                    format!(
                        "{}..{}",
                        Self::format_object(start),
                        Self::format_object(end)
                    )
                }
            }
            Object::Exception(e) => {
                let e_borrowed = e.borrow();
                format!("<Exception: {}>", e_borrowed.message)
            }
            Object::Set(s) => {
                let s_borrowed = s.borrow();
                format!("<Set: {} items>", s_borrowed.len())
            }
            Object::Result(r) => match r {
                Ok(v) => format!("<Ok: {}>", Self::format_object(v)),
                Err(e) => format!("<Err: {}>", Self::format_object(e)),
            },
            Object::Binding(binding) => {
                format!("<Binding with {} vars>", binding.variables.len())
            }
            Object::CompiledFunction(func) => format!("{}", func),
            Object::Regex(pattern, flags) => {
                if flags.is_empty() {
                    format!("/{}/", pattern)
                } else {
                    format!("/{}/{}", pattern, flags)
                }
            }
        }
    }

    /// Format an error for display.
    pub fn format_error(err: &MetorexError) -> String {
        err.to_string()
    }
}

impl Default for ReplCore {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Repl — thin interactive wrapper around ReplCore
// ─────────────────────────────────────────────────────────────────────────────

pub struct Repl {
    core: ReplCore,
    editor: DefaultEditor,
}

impl Repl {
    /// Create a new REPL instance.
    pub fn new() -> RustylineResult<Self> {
        let editor = DefaultEditor::new()?;
        Ok(Self {
            core: ReplCore::new(),
            editor,
        })
    }

    /// Start the REPL loop.
    pub fn run(&mut self) -> RustylineResult<()> {
        println!("{}", ReplCore::banner());

        loop {
            let prompt = self.core.prompt();

            match self.editor.readline(prompt) {
                Ok(line) => {
                    let _ = self.editor.add_history_entry(&line);

                    match self.core.process_line(&line) {
                        LineResult::Continue => {}
                        LineResult::Exit => {
                            println!("Goodbye!");
                            return Ok(());
                        }
                        LineResult::Value(text) => println!("{}", text),
                        LineResult::Error(text) => eprintln!("{}", text),
                        LineResult::Incomplete => {}
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    self.core.clear_buffer();
                }
                Err(ReadlineError::Eof) => {
                    println!("exit");
                    return Ok(());
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    return Err(err);
                }
            }
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new().expect("Failed to initialize REPL")
    }
}
