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
    compile_prism();
    compile_nkf();
}

/// nkf's C extension, which `require "nkf"` starts with `Init_nkf`, compiled
/// against the headers in `include` from the copy scripts/vendor_gems.rb
/// makes.
fn compile_nkf() {
    println!("cargo:rerun-if-changed=vendor/nkf");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let object = out.join("nkf.o");
    let status = std::process::Command::new(&compiler)
        .args([
            "-O2",
            "-fPIC",
            "-w",
            "-I",
            "include",
            "-I",
            "vendor/nkf",
            "-c",
        ])
        .arg("vendor/nkf/nkf.c")
        .arg("-o")
        .arg(&object)
        .status()
        .unwrap_or_else(|error| panic!("{compiler} could not start: {error}"));
    assert!(status.success(), "{compiler} failed on vendor/nkf/nkf.c");
    archive(&out, "nkf", &[object]);
}

/// Bundle object files into `lib<name>.a` in `out` and link it in.
fn archive(out: &Path, name: &str, objects: &[PathBuf]) {
    let archive = out.join(format!("lib{name}.a"));
    let _ = std::fs::remove_file(&archive);
    let status = std::process::Command::new("ar")
        .arg("crs")
        .arg(&archive)
        .args(objects)
        .status()
        .expect("ar ran");
    assert!(status.success(), "ar failed on {}", archive.display());
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static={name}");
}

const PRISM: &str = "vendor/prism";

fn c_sources_under(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            c_sources_under(&path, found);
        } else if path.extension().is_some_and(|held| held == "c") {
            found.push(path);
        }
    }
}

/// Prism's parser, which `require "prism"` reaches through native
/// functions, compiled from the copy scripts/vendor_prism.rb makes and linked
/// in as a static library.
fn compile_prism() {
    println!("cargo:rerun-if-changed={PRISM}");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let include = format!("{PRISM}/include");
    let mut sources = Vec::new();
    c_sources_under(&Path::new(PRISM).join("src"), &mut sources);
    sources.sort();
    let compiling: Vec<(PathBuf, std::process::Child)> = sources
        .iter()
        .map(|source| {
            let object = out.join(
                source
                    .strip_prefix(PRISM)
                    .expect("sources sit under the prism copy")
                    .to_string_lossy()
                    .replace(['/', '\\'], "_")
                    .replace(".c", ".o"),
            );
            let child = std::process::Command::new(&compiler)
                .args(["-std=c99", "-O2", "-fPIC", "-fvisibility=hidden", "-I"])
                .arg(&include)
                .arg("-c")
                .arg(source)
                .arg("-o")
                .arg(&object)
                .spawn()
                .unwrap_or_else(|error| panic!("{compiler} could not start: {error}"));
            (object, child)
        })
        .collect();
    let mut objects = Vec::new();
    for (object, mut child) in compiling {
        let status = child.wait().expect("the compiler ran");
        assert!(
            status.success(),
            "{compiler} failed on {}",
            object.display()
        );
        objects.push(object);
    }
    archive(&out, "prism", &objects);
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
