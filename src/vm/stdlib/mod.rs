// The libraries metorex ships with, held in the binary so `require` finds
// them without a directory on the load path.

/// The file a library metorex carries says its code was written in.
pub(crate) fn embedded_library_file(name: &str) -> String {
    format!(
        "{}{}.rb",
        EMBEDDED_LIBRARY_PREFIX,
        name.strip_suffix(".rb").unwrap_or(name)
    )
}

const EMBEDDED_LIBRARY_PREFIX: &str = "<metorex>/";

/// The libraries metorex carries that Ruby writes in C, as `ext/` holds them.
const C_EXTENSIONS: &[&str] = &[
    "bigdecimal",
    "coverage",
    "date",
    "etc",
    "fcntl",
    "fiddle",
    "io/console",
    "io/nonblock",
    "monitor",
    "objspace",
    "rbconfig/sizeof",
    "stringio",
    "strscan",
    "syslog",
    "zlib",
];

/// Whether code written in `file` stands for code Ruby writes in C: the
/// core library metorex loads at startup, or a library Ruby ships as a C
/// extension.
pub(crate) fn written_for_c(file: &str) -> bool {
    if file.starts_with("<internal:") {
        return true;
    }
    file.strip_prefix(EMBEDDED_LIBRARY_PREFIX)
        .and_then(|named| named.strip_suffix(".rb"))
        .is_some_and(|named| C_EXTENSIONS.contains(&named))
}

/// The source of a library metorex carries, or None for a name it does not
/// have. A file on the load path wins over one of these.
pub(crate) fn embedded_library(name: &str) -> Option<&'static str> {
    let trimmed = name.strip_suffix(".rb").unwrap_or(name);
    match trimmed {
        "base64" => Some(include_str!("base64.rb")),
        "bigdecimal" | "bigdecimal/util" => Some(include_str!("bigdecimal.rb")),
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
        "rubygems/text" => Some(include_str!("rubygems/text.rb")),
        "rubygems/user_interaction" => Some(include_str!("rubygems/user_interaction.rb")),
        "rubygems/gemcutter_utilities" => Some(include_str!("rubygems/gemcutter_utilities.rb")),
        "rubygems/command_manager" => Some(include_str!("rubygems/command_manager.rb")),
        "rubygems/command" => Some(include_str!("rubygems/command.rb")),
        "rubygems/safe_yaml" => Some(include_str!("rubygems/safe_yaml.rb")),
        "rubygems/commands/owner_command" => {
            Some(include_str!("rubygems/commands/owner_command.rb"))
        }
        "optparse" | "optionparser" => Some(include_str!("optparse.rb")),
        "random/formatter" => Some(include_str!("random_formatter.rb")),
        "socket" => Some(include_str!("socket.rb")),
        "erb" => Some(include_str!("erb.rb")),
        "abbrev" => Some(include_str!("abbrev.rb")),
        "etc" => Some(include_str!("etc.rb")),
        "mkmf" => Some(include_str!("mkmf.rb")),
        "pp" => Some(include_str!("pp.rb")),
        "fcntl" => Some(include_str!("fcntl.rb")),
        "expect" => Some(include_str!("expect.rb")),
        "fiber" => Some(include_str!("fiber.rb")),
        "open-uri" => Some(include_str!("open_uri.rb")),
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
        "fileutils" => Some(include_str!("fileutils.rb")),
        "tmpdir" => Some(include_str!("tmpdir.rb")),
        "tempfile" => Some(include_str!("tempfile.rb")),
        // Every name the digest library is reached by loads the one
        // file, which carries all of the algorithms metorex has.
        "digest" | "digest/md5" | "digest/sha1" | "digest/sha2" | "digest/bubblebabble" => {
            Some(include_str!("digest.rb"))
        }
        "matrix" => Some(include_str!("matrix.rb")),
        "drb" | "drb/drb" => Some(include_str!("drb.rb")),
        "fiddle" => Some(include_str!("fiddle.rb")),
        "irb" => Some(include_str!("irb.rb")),
        "json" => Some(include_str!("json.rb")),
        "objspace" => Some(include_str!("objspace.rb")),
        "objspace/trace" => Some(include_str!("objspace_trace.rb")),
        "observer" => Some(include_str!("observer.rb")),
        "ostruct" => Some(include_str!("ostruct.rb")),
        "pathname" => Some(include_str!("pathname.rb")),
        "prime" => Some(include_str!("prime.rb")),
        "ripper" => Some(include_str!("ripper.rb")),
        // The scanner and the grammar, which Ruby writes in C, load with
        // the core class that dispatches their events.
        "ripper/core" => Some(concat!(
            include_str!("ripper_core.rb"),
            include_str!("ripper_scanner.rb"),
            include_str!("ripper_grammar.rb")
        )),
        "ripper/filter" => Some(include_str!("ripper_filter.rb")),
        "ripper/lexer" => Some(include_str!("ripper_lexer.rb")),
        "ripper/sexp" => Some(include_str!("ripper_sexp.rb")),
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
