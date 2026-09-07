// The libraries metorex ships with, held in the binary so `require` finds
// them without a directory on the load path.

/// The source of a library metorex carries, or None for a name it does not
/// have. A file on the load path wins over one of these.
pub(crate) fn embedded_library(name: &str) -> Option<&'static str> {
    let trimmed = name.strip_suffix(".rb").unwrap_or(name);
    match trimmed {
        "base64" => Some(include_str!("base64.rb")),
        "abbrev" => Some(include_str!("abbrev.rb")),
        "io/console" => Some(include_str!("io_console.rb")),
        "date" => Some(include_str!("date.rb")),
        "matrix" => Some(include_str!("matrix.rb")),
        "observer" => Some(include_str!("observer.rb")),
        "ostruct" => Some(include_str!("ostruct.rb")),
        "prime" => Some(include_str!("prime.rb")),
        "securerandom" => Some(include_str!("securerandom.rb")),
        "shellwords" => Some(include_str!("shellwords.rb")),
        "singleton" => Some(include_str!("singleton.rb")),
        "stringio" => Some(include_str!("stringio.rb")),
        "strscan" => Some(include_str!("strscan.rb")),
        _ => None,
    }
}
