// The libraries metorex ships with, held in the binary so `require` finds
// them without a directory on the load path.

/// The source of a library metorex carries, or None for a name it does not
/// have. A file on the load path wins over one of these.
pub(crate) fn embedded_library(name: &str) -> Option<&'static str> {
    let trimmed = name.strip_suffix(".rb").unwrap_or(name);
    match trimmed {
        "base64" => Some(include_str!("base64.rb")),
        "bigdecimal" => Some(include_str!("bigdecimal.rb")),
        "zlib" => Some(include_str!("zlib.rb")),
        "open3" => Some(include_str!("open3.rb")),
        "syslog" => Some(include_str!("syslog.rb")),
        "openssl" => Some(include_str!("openssl.rb")),
        "cgi" | "cgi/escape" | "cgi/util" => Some(include_str!("cgi.rb")),
        "net/http" | "net/https" | "net/protocol" => Some(include_str!("net_http.rb")),
        "net/ftp" => Some(include_str!("net_ftp.rb")),
        "English" | "english" => Some(include_str!("english.rb")),
        "io/nonblock" => Some(include_str!("io_nonblock.rb")),
        "rbconfig" => Some(include_str!("rbconfig.rb")),
        "rbconfig/sizeof" => Some(include_str!("rbconfig_sizeof.rb")),
        "yaml" | "psych" => Some(include_str!("yaml.rb")),
        "resolv" => Some(include_str!("resolv.rb")),
        "rubygems" => Some(include_str!("rubygems.rb")),
        "optparse" | "optionparser" => Some(include_str!("optparse.rb")),
        "random/formatter" => Some(include_str!("random_formatter.rb")),
        "socket" => Some(include_str!("socket.rb")),
        "erb" => Some(include_str!("erb.rb")),
        "bigdecimal/util" => Some(include_str!("bigdecimal.rb")),
        "abbrev" => Some(include_str!("abbrev.rb")),
        "etc" => Some(include_str!("etc.rb")),
        "find" => Some(include_str!("find.rb")),
        "getoptlong" => Some(include_str!("getoptlong.rb")),
        "io/console" => Some(include_str!("io_console.rb")),
        "ipaddr" => Some(include_str!("ipaddr.rb")),
        "coverage" => Some(include_str!("coverage.rb")),
        "csv" => Some(include_str!("csv.rb")),
        "date" => Some(include_str!("date.rb")),
        "delegate" => Some(include_str!("delegate.rb")),
        "weakref" => Some(include_str!("weakref.rb")),
        "logger" => Some(include_str!("logger.rb")),
        "monitor" => Some(include_str!("monitor.rb")),
        "tmpdir" => Some(include_str!("tmpdir.rb")),
        "tempfile" => Some(include_str!("tempfile.rb")),
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
