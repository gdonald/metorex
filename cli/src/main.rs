// Metorex CLI
// Command-line interface for the Metorex programming language

fn main() {
    println!("Hello, Metorex!");
    println!("Core version: {}", metorex_core::core_version());
    println!("Runtime version: {}", metorex_runtime::runtime_version());
}
