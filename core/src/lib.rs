// Metorex Core Library
// Contains core abstractions, types, and fundamental building blocks

pub mod error;

pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
