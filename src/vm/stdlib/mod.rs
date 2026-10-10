// The libraries metorex ships with, held in the binary so `require` finds
// them without a directory on the load path.

mod gem_libraries;
mod prism_libraries;
use gem_libraries::GEM_LIBRARIES;
use prism_libraries::PRISM_LIBRARIES;

/// The file a library metorex carries says its code was written in.
pub(crate) fn embedded_library_file(name: &str) -> String {
    // A file MRI itself compiles in names itself the way MRI does.
    if name.starts_with("<internal:") {
        return name.to_string();
    }
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
    (&["net/http"], include_str!("net/http.rb")),
    (&["net/https"], include_str!("net/https.rb")),
    (&["net/protocol"], include_str!("net/protocol.rb")),
    (
        &["net/http/exceptions"],
        include_str!("net/http/exceptions.rb"),
    ),
    (&["net/http/header"], include_str!("net/http/header.rb")),
    (
        &["net/http/generic_request"],
        include_str!("net/http/generic_request.rb"),
    ),
    (&["net/http/request"], include_str!("net/http/request.rb")),
    (&["net/http/requests"], include_str!("net/http/requests.rb")),
    (&["net/http/response"], include_str!("net/http/response.rb")),
    (
        &["net/http/responses"],
        include_str!("net/http/responses.rb"),
    ),
    (&["net/http/status"], include_str!("net/http/status.rb")),
    (
        &["net/http/proxy_delta"],
        include_str!("net/http/proxy_delta.rb"),
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
    (&["erb/version"], include_str!("erb/version.rb")),
    (&["erb/compiler"], include_str!("erb/compiler.rb")),
    (&["erb/def_method"], include_str!("erb/def_method.rb")),
    (&["erb/util"], include_str!("erb/util.rb")),
    (&["abbrev"], include_str!("abbrev.rb")),
    (&["etc"], include_str!("etc.rb")),
    (&["mkmf"], include_str!("mkmf.rb")),
    (&["pp"], include_str!("pp.rb")),
    (&["prettyprint"], include_str!("prettyprint.rb")),
    (&["fcntl"], include_str!("fcntl.rb")),
    (&["expect"], include_str!("expect.rb")),
    (&["fiber"], include_str!("fiber.rb")),
    (&["open-uri"], include_str!("open_uri.rb")),
    (&["find"], include_str!("find.rb")),
    (&["getoptlong"], include_str!("getoptlong.rb")),
    (&["io/console"], include_str!("io_console.rb")),
    (&["io/console/size"], include_str!("io_console_size.rb")),
    (&["ipaddr"], include_str!("ipaddr.rb")),
    (&["coverage"], include_str!("coverage.rb")),
    (&["csv"], include_str!("csv.rb")),
    (&["date"], include_str!("date.rb")),
    (&["delegate"], include_str!("delegate.rb")),
    (
        &[
            "error_highlight",
            "error_highlight/base",
            "error_highlight/core_ext",
            "error_highlight/formatter",
            "error_highlight/version",
        ],
        include_str!("error_highlight.rb"),
    ),
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
    (&["syntax_suggest"], include_str!("syntax_suggest.rb")),
    (
        &["syntax_suggest/core_ext"],
        include_str!("syntax_suggest_core_ext.rb"),
    ),
    (
        &[
            "syntax_suggest/api",
            "syntax_suggest/around_block_scan",
            "syntax_suggest/block_expand",
            "syntax_suggest/capture/before_after_keyword_ends",
            "syntax_suggest/capture/falling_indent_lines",
            "syntax_suggest/capture_code_context",
            "syntax_suggest/clean_document",
            "syntax_suggest/cli",
            "syntax_suggest/code_block",
            "syntax_suggest/code_frontier",
            "syntax_suggest/code_line",
            "syntax_suggest/code_search",
            "syntax_suggest/display_code_with_line_numbers",
            "syntax_suggest/display_invalid_blocks",
            "syntax_suggest/explain_syntax",
            "syntax_suggest/left_right_lex_count",
            "syntax_suggest/lex_all",
            "syntax_suggest/lex_value",
            "syntax_suggest/mini_stringio",
            "syntax_suggest/parse_blocks_from_indent_line",
            "syntax_suggest/pathname_from_message",
            "syntax_suggest/priority_engulf_queue",
            "syntax_suggest/priority_queue",
            "syntax_suggest/ripper_errors",
            "syntax_suggest/scan_history",
            "syntax_suggest/unvisited_lines",
            "syntax_suggest/version",
        ],
        include_str!("syntax_suggest_api.rb"),
    ),
    // What prism's C extension defines, over the parser build.rs compiles in.
    (&["prism/prism"], include_str!("prism_backend.rb")),
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
    (&["json"], include_str!("json.rb")),
    (&["objspace"], include_str!("objspace.rb")),
    (&["objspace/trace"], include_str!("objspace_trace.rb")),
    (&["observer"], include_str!("observer.rb")),
    (&["ostruct"], include_str!("ostruct.rb")),
    (&["pathname"], include_str!("pathname.rb")),
    (
        &["<internal:pathname_builtin>"],
        include_str!("pathname_builtin.rb"),
    ),
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
    // MRI's parser tables, read when a syntax error is worded.
    (
        &["<internal:parser_tables>"],
        include_str!("parser_tables.rb"),
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
    (&["uri"], include_str!("uri_vendored.rb")),
    (&["uri/common"], include_str!("uri/common.rb")),
    (&["uri/file"], include_str!("uri/file.rb")),
    (&["uri/ftp"], include_str!("uri/ftp.rb")),
    (&["uri/generic"], include_str!("uri/generic.rb")),
    (&["uri/http"], include_str!("uri/http.rb")),
    (&["uri/https"], include_str!("uri/https.rb")),
    (&["uri/ldap"], include_str!("uri/ldap.rb")),
    (&["uri/ldaps"], include_str!("uri/ldaps.rb")),
    (&["uri/mailto"], include_str!("uri/mailto.rb")),
    (
        &["uri/rfc2396_parser"],
        include_str!("uri/rfc2396_parser.rb"),
    ),
    (
        &["uri/rfc3986_parser"],
        include_str!("uri/rfc3986_parser.rb"),
    ),
    (&["uri/version"], include_str!("uri/version.rb")),
    (&["uri/ws"], include_str!("uri/ws.rb")),
    (&["uri/wss"], include_str!("uri/wss.rb")),
];

/// The source of a library metorex carries, or None for a name it does not
/// have. A file on the load path wins over one of these.
pub(crate) fn embedded_library(name: &str) -> Option<&'static str> {
    let trimmed = name.strip_suffix(".rb").unwrap_or(name);
    EMBEDDED_LIBRARIES
        .iter()
        .find(|(names, _)| names.contains(&trimmed))
        .map(|(_, source)| *source)
        .or_else(|| {
            PRISM_LIBRARIES
                .iter()
                .chain(GEM_LIBRARIES)
                .find(|(named, _)| *named == trimmed)
                .map(|(_, source)| *source)
        })
}

/// Every name `require` finds a library metorex carries under.
pub(crate) fn embedded_library_names() -> impl Iterator<Item = &'static str> {
    EMBEDDED_LIBRARIES
        .iter()
        .flat_map(|(names, _)| names.iter().copied())
        .chain(
            PRISM_LIBRARIES
                .iter()
                .chain(GEM_LIBRARIES)
                .map(|(named, _)| *named),
        )
}
