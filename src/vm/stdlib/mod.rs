// The libraries metorex ships with, held in the binary so `require` finds
// them without a directory on the load path.

/// The source of a library metorex carries, or None for a name it does not
/// have. A file on the load path wins over one of these.
pub(crate) fn embedded_library(name: &str) -> Option<&'static str> {
    let trimmed = name.strip_suffix(".rb").unwrap_or(name);
    match trimmed {
        "base64" => Some(include_str!("base64.rb")),
        "abbrev" => Some(include_str!("abbrev.rb")),
        "etc" => Some(include_str!("etc.rb")),
        "find" => Some(include_str!("find.rb")),
        "getoptlong" => Some(include_str!("getoptlong.rb")),
        "io/console" => Some(include_str!("io_console.rb")),
        "ipaddr" => Some(include_str!("ipaddr.rb")),
        "coverage" => Some(include_str!("coverage.rb")),
        "csv" => Some(include_str!("csv.rb")),
        "date" => Some(include_str!("date.rb")),
        // Every name the digest library is reached by loads the one
        // file, which carries all of the algorithms metorex has.
        "digest" | "digest/md5" | "digest/sha1" | "digest/sha2" | "digest/bubblebabble" => {
            Some(include_str!("digest.rb"))
        }
        "matrix" => Some(include_str!("matrix.rb")),
        "objspace" => Some(include_str!("objspace.rb")),
        "observer" => Some(include_str!("observer.rb")),
        "ostruct" => Some(include_str!("ostruct.rb")),
        "pathname" => Some(include_str!("pathname.rb")),
        "prime" => Some(include_str!("prime.rb")),
        "securerandom" => Some(include_str!("securerandom.rb")),
        "shellwords" => Some(include_str!("shellwords.rb")),
        "singleton" => Some(include_str!("singleton.rb")),
        "stringio" => Some(include_str!("stringio.rb")),
        "strscan" => Some(include_str!("strscan.rb")),
        "time" => Some(include_str!("time.rb")),
        "timeout" => Some(include_str!("timeout.rb")),
        "uri" => Some(include_str!("uri.rb")),
        _ => None,
    }
}
