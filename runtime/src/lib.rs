// Metorex Runtime Library
// Contains the runtime execution environment, actor management, and message passing

pub fn runtime_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
