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

pub struct Repl {
    vm: VirtualMachine,
    editor: DefaultEditor,
    buffer: String,
}

impl Repl {
    /// Create a new REPL instance
    pub fn new() -> RustylineResult<Self> {
        let editor = DefaultEditor::new()?;
        Ok(Self {
            vm: VirtualMachine::new(),
            editor,
            buffer: String::new(),
        })
    }

    /// Start the REPL loop
    pub fn run(&mut self) -> RustylineResult<()> {
        println!("{}", BANNER);
        println!("Metorex REPL v{}", env!("CARGO_PKG_VERSION"));
        println!("Type .help for more information, .exit to quit");
        println!();

        loop {
            let prompt = if self.buffer.is_empty() {
                PROMPT
            } else {
                CONTINUATION_PROMPT
            };

            match self.editor.readline(prompt) {
                Ok(line) => {
                    // Add to history
                    let _ = self.editor.add_history_entry(&line);

                    // Handle special commands
                    if self.buffer.is_empty() && line.trim().starts_with('.') {
                        if self.handle_command(&line) {
                            return Ok(());
                        }
                        continue;
                    }

                    // Add line to buffer
                    if !self.buffer.is_empty() {
                        self.buffer.push('\n');
                    }
                    self.buffer.push_str(&line);

                    // Try to evaluate the buffer
                    if self.should_evaluate() {
                        self.evaluate_buffer();
                        self.buffer.clear();
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl-C - clear buffer and continue
                    println!("^C");
                    self.buffer.clear();
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl-D - exit
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

    /// Handle special REPL commands
    fn handle_command(&mut self, line: &str) -> bool {
        let cmd = line.trim();

        match cmd {
            ".exit" | ".quit" => {
                println!("Goodbye!");
                return true;
            }
            ".help" => {
                self.print_help();
            }
            ".clear" => {
                print!("\x1B[2J\x1B[1;1H"); // ANSI escape codes to clear screen
                println!("{}", BANNER);
                println!("Metorex REPL v{}", env!("CARGO_PKG_VERSION"));
                println!("Type .help for more information, .exit to quit");
                println!();
            }
            ".reset" => {
                self.vm = VirtualMachine::new();
                println!("VM state reset");
            }
            _ => {
                eprintln!("Unknown command: {}", cmd);
                eprintln!("Type .help for available commands");
            }
        }

        false
    }

    /// Print help information
    fn print_help(&self) {
        println!("Metorex REPL Commands:");
        println!("  .help       Show this help message");
        println!("  .exit       Exit the REPL (or Ctrl-D)");
        println!("  .quit       Alias for .exit");
        println!("  .clear      Clear the screen");
        println!("  .reset      Reset the VM state");
        println!();
        println!("Keyboard shortcuts:");
        println!("  Ctrl-C      Clear current input buffer");
        println!("  Ctrl-D      Exit the REPL");
        println!();
        println!("Multi-line input:");
        println!("  The REPL automatically detects incomplete expressions");
        println!("  and prompts for continuation with '..'");
        println!();
    }

    /// Determine if the current buffer should be evaluated
    fn should_evaluate(&self) -> bool {
        let trimmed = self.buffer.trim();

        // Empty input
        if trimmed.is_empty() {
            return true;
        }

        // Try to parse and see if we get a complete statement
        let lexer = Lexer::new(&self.buffer);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        match parser.parse() {
            Ok(_) => true, // Successfully parsed, ready to evaluate
            Err(errors) => {
                // Check if error indicates incomplete input
                for error in &errors {
                    let error_msg = error.to_string().to_lowercase();
                    if error_msg.contains("unexpected end of input")
                        || error_msg.contains("expected 'end'")
                        || error_msg.contains("unclosed")
                        || error_msg.contains("incomplete")
                    {
                        return false; // Need more input
                    }
                }
                // Other parse errors - try to evaluate anyway to show the error
                true
            }
        }
    }

    /// Evaluate the current buffer
    fn evaluate_buffer(&mut self) {
        // Tokenize
        let lexer = Lexer::new(&self.buffer);
        let tokens = lexer.tokenize();

        // Parse
        let mut parser = Parser::new(tokens);
        let program = match parser.parse() {
            Ok(prog) => prog,
            Err(errors) => {
                for err in errors {
                    eprintln!("Parse error: {}", err);
                }
                return;
            }
        };

        // Execute and display result
        match self.vm.execute_program(&program) {
            Ok(Some(result)) => {
                // Display non-nil results
                if !matches!(result, Object::Nil) {
                    println!("=> {}", Self::format_object(&result));
                }
            }
            Ok(None) => {
                // No result (e.g., statements like assignments)
            }
            Err(err) => {
                eprintln!("Runtime error: {}", self.format_error(&err));
            }
        }
    }

    /// Format an object for display
    pub fn format_object(obj: &Object) -> String {
        match obj {
            Object::Nil => "nil".to_string(),
            Object::Bool(b) => b.to_string(),
            Object::Int(i) => i.to_string(),
            Object::Float(f) => {
                // Format float nicely
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{:.1}", f)
                } else {
                    f.to_string()
                }
            }
            Object::String(s) => format!("\"{}\"", s),
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
                entries.sort(); // Sort for consistent display
                format!("{{{}}}", entries.join(", "))
            }
            Object::Block { .. } => "<Block>".to_string(),
            Object::Method { .. } => "<Method>".to_string(),
            Object::NativeFunction(name) => format!("<NativeFunction: {}>", name),
            Object::Class(class) => format!("<Class: {}>", class.name()),
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
        }
    }

    /// Format an error for display
    fn format_error(&self, err: &MetorexError) -> String {
        err.to_string()
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new().expect("Failed to initialize REPL")
    }
}
