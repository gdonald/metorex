// The binary exports its `rb_*` functions so a C extension it loads with
// `dlopen` resolves its references against them.
//
// `include/ruby/internal/abi.h` carries a digest of the other headers. The
// spec suite rebuilds an extension older than that file, so any change to
// the headers rebuilds every extension compiled against them.

use std::path::{Path, PathBuf};

const ABI_HEADER: &str = "include/ruby/internal/abi.h";

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        println!("cargo:rustc-link-arg-bins=-Wl,-export_dynamic");
    } else {
        println!("cargo:rustc-link-arg-bins=-rdynamic");
    }
    println!("cargo:rerun-if-changed=include");
    write_abi_header();
}

fn headers_under(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            headers_under(&path, found);
        } else if path.extension().is_some_and(|held| held == "h") && path != Path::new(ABI_HEADER)
        {
            found.push(path);
        }
    }
}

/// FNV-1a, which gives the same digest on every toolchain.
fn digest(bytes: &[u8], mut hash: u64) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn write_abi_header() {
    let mut headers = Vec::new();
    headers_under(Path::new("include"), &mut headers);
    headers.sort();
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for header in &headers {
        hash = digest(header.to_string_lossy().as_bytes(), hash);
        hash = digest(&std::fs::read(header).unwrap_or_default(), hash);
    }
    let contents = format!(
        "#ifndef RUBY_ABI_H\n#define RUBY_ABI_H 1\n\n/* Written by build.rs from a digest of the other headers. */\n#define RUBY_ABI_VERSION 0x{hash:016x}\n\n#endif\n"
    );
    if std::fs::read_to_string(ABI_HEADER).ok().as_deref() != Some(contents.as_str()) {
        std::fs::create_dir_all("include/ruby/internal").expect("include/ruby is writable");
        std::fs::write(ABI_HEADER, contents).expect("include/ruby is writable");
    }
}
