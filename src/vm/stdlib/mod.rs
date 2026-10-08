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

/// The libraries metorex carries: the names `require` reaches each by, and
/// its source.
const EMBEDDED_LIBRARIES: &[(&[&str], &str)] = &[
    (&["base64"], include_str!("base64.rb")),
    (
        &["bigdecimal", "bigdecimal/util"],
        include_str!("bigdecimal.rb"),
    ),
    (&["zlib"], include_str!("zlib.rb")),
    (&["open3"], include_str!("open3.rb")),
    (&["syslog"], include_str!("syslog.rb")),
    (&["openssl"], include_str!("openssl.rb")),
    (&["cgi", "cgi/escape", "cgi/util"], include_str!("cgi.rb")),
    (
        &["net/http", "net/https", "net/protocol"],
        include_str!("net_http.rb"),
    ),
    (&["net/ftp"], include_str!("net_ftp.rb")),
    (&["English", "english"], include_str!("english.rb")),
    (&["io/nonblock"], include_str!("io_nonblock.rb")),
    (&["rbconfig"], include_str!("rbconfig.rb")),
    (&["rbconfig/sizeof"], include_str!("rbconfig_sizeof.rb")),
    (&["yaml", "psych"], include_str!("yaml.rb")),
    (&["resolv"], include_str!("resolv.rb")),
    (&["rubygems"], include_str!("rubygems.rb")),
    (&["rubygems/text"], include_str!("rubygems/text.rb")),
    (
        &["rubygems/user_interaction"],
        include_str!("rubygems/user_interaction.rb"),
    ),
    (
        &["rubygems/gemcutter_utilities"],
        include_str!("rubygems/gemcutter_utilities.rb"),
    ),
    (
        &["rubygems/command_manager"],
        include_str!("rubygems/command_manager.rb"),
    ),
    (&["rubygems/command"], include_str!("rubygems/command.rb")),
    (
        &["rubygems/safe_yaml"],
        include_str!("rubygems/safe_yaml.rb"),
    ),
    (
        &["rubygems/commands/owner_command"],
        include_str!("rubygems/commands/owner_command.rb"),
    ),
    (&["optparse", "optionparser"], include_str!("optparse.rb")),
    (&["random/formatter"], include_str!("random_formatter.rb")),
    (&["socket"], include_str!("socket.rb")),
    (&["erb"], include_str!("erb.rb")),
    (&["abbrev"], include_str!("abbrev.rb")),
    (&["etc"], include_str!("etc.rb")),
    (&["mkmf"], include_str!("mkmf.rb")),
    (&["pp"], include_str!("pp.rb")),
    (&["fcntl"], include_str!("fcntl.rb")),
    (&["expect"], include_str!("expect.rb")),
    (&["fiber"], include_str!("fiber.rb")),
    (&["open-uri"], include_str!("open_uri.rb")),
    (&["find"], include_str!("find.rb")),
    (&["getoptlong"], include_str!("getoptlong.rb")),
    (&["io/console"], include_str!("io_console.rb")),
    (&["ipaddr"], include_str!("ipaddr.rb")),
    (&["coverage"], include_str!("coverage.rb")),
    (&["csv"], include_str!("csv.rb")),
    (&["date"], include_str!("date.rb")),
    (&["delegate"], include_str!("delegate.rb")),
    (
        &[
            "did_you_mean",
            "did_you_mean/core_ext/name_error",
            "did_you_mean/formatter",
            "did_you_mean/jaro_winkler",
            "did_you_mean/levenshtein",
            "did_you_mean/spell_checker",
            "did_you_mean/spell_checkers/key_error_checker",
            "did_you_mean/spell_checkers/method_name_checker",
            "did_you_mean/spell_checkers/name_error_checkers",
            "did_you_mean/spell_checkers/null_checker",
            "did_you_mean/spell_checkers/pattern_key_name_checker",
            "did_you_mean/spell_checkers/require_path_checker",
            "did_you_mean/tree_spell_checker",
            "did_you_mean/version",
        ],
        include_str!("did_you_mean.rb"),
    ),
    (&["weakref"], include_str!("weakref.rb")),
    (&["logger"], include_str!("logger.rb")),
    (&["monitor"], include_str!("monitor.rb")),
    (
        &["forwardable", "forwardable/impl"],
        include_str!("forwardable.rb"),
    ),
    (&["tsort"], include_str!("tsort.rb")),
    (&["io/wait"], include_str!("io_wait.rb")),
    (&["pty"], include_str!("pty.rb")),
    (&["continuation"], include_str!("continuation.rb")),
    (&["pstore"], include_str!("pstore.rb")),
    (&["benchmark"], include_str!("benchmark.rb")),
    (&["un"], include_str!("un.rb")),
    (&["fileutils"], include_str!("fileutils.rb")),
    (&["tmpdir"], include_str!("tmpdir.rb")),
    (&["tempfile"], include_str!("tempfile.rb")),
    // Every name the digest library is reached by loads the one
    // file, which carries all of the algorithms metorex has.
    (
        &[
            "digest",
            "digest/md5",
            "digest/sha1",
            "digest/sha2",
            "digest/rmd160",
            "digest/bubblebabble",
        ],
        include_str!("digest.rb"),
    ),
    (&["matrix"], include_str!("matrix.rb")),
    (&["drb", "drb/drb"], include_str!("drb.rb")),
    (&["fiddle"], include_str!("fiddle.rb")),
    (&["irb"], include_str!("irb.rb")),
    (&["irb/color"], include_str!("irb_color.rb")),
    (&["json"], include_str!("json.rb")),
    (&["objspace"], include_str!("objspace.rb")),
    (&["objspace/trace"], include_str!("objspace_trace.rb")),
    (&["observer"], include_str!("observer.rb")),
    (&["ostruct"], include_str!("ostruct.rb")),
    (&["pathname"], include_str!("pathname.rb")),
    (&["prime"], include_str!("prime.rb")),
    (&["ripper"], include_str!("ripper.rb")),
    // The scanner and the grammar, which Ruby writes in C, load with
    // the core class that dispatches their events.
    (
        &["ripper/core"],
        concat!(
            include_str!("ripper_core.rb"),
            include_str!("ripper_scanner.rb"),
            include_str!("ripper_grammar.rb")
        ),
    ),
    (&["ripper/filter"], include_str!("ripper_filter.rb")),
    (&["ripper/lexer"], include_str!("ripper_lexer.rb")),
    (&["ripper/sexp"], include_str!("ripper_sexp.rb")),
    (&["securerandom"], include_str!("securerandom.rb")),
    (&["shellwords"], include_str!("shellwords.rb")),
    (&["singleton"], include_str!("singleton.rb")),
    (&["stringio"], include_str!("stringio.rb")),
    (&["strscan"], include_str!("strscan.rb")),
    (&["time"], include_str!("time.rb")),
    (&["timeout"], include_str!("timeout.rb")),
    (&["uri"], include_str!("uri.rb")),
];

/// The source of a library metorex carries, or None for a name it does not
/// have. A file on the load path wins over one of these.
pub(crate) fn embedded_library(name: &str) -> Option<&'static str> {
    let trimmed = name.strip_suffix(".rb").unwrap_or(name);
    EMBEDDED_LIBRARIES
        .iter()
        .find(|(names, _)| names.contains(&trimmed))
        .map(|(_, source)| *source)
}

/// Every name `require` finds a library metorex carries under.
pub(crate) fn embedded_library_names() -> impl Iterator<Item = &'static str> {
    EMBEDDED_LIBRARIES
        .iter()
        .flat_map(|(names, _)| names.iter().copied())
}
