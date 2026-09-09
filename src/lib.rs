// Metorex Programming Language
// A modern, Ruby-inspired language with powerful metaprogramming capabilities

pub mod ast;
pub mod builtin_classes;
pub mod bytecode;
pub mod callable;
pub mod class;
pub mod compiler;
pub mod environment;
pub mod error;
pub mod file_loader;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod repl;
pub mod resolver;
pub mod runtime;
pub mod scope;
pub mod test_discovery;
pub mod vm;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The Ruby release metorex reports as its own. It names the release the
/// vendored ruby/spec suite is written for, which decides which of the
/// suite's version guards open.
pub const DEFAULT_RUBY_VERSION: &str = "4.0.1";

/// The Ruby release to report, which `METOREX_RUBY_VERSION` may name instead
/// so the spec suite can be run against the guards of another release.
/// The platform Ruby would name this machine, which is an architecture and
/// an operating system joined by a dash. The spec suite's platform guards
/// read it, and they look for `darwin` rather than `macos`.
pub fn reported_ruby_platform() -> String {
    let architecture = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    let system = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    format!("{architecture}-{system}")
}

pub fn reported_ruby_version() -> String {
    match std::env::var("METOREX_RUBY_VERSION") {
        Ok(named) if !named.trim().is_empty() => named.trim().to_string(),
        _ => DEFAULT_RUBY_VERSION.to_string(),
    }
}
