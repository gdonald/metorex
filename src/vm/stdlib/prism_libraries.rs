// Written by scripts/vendor_prism.rb from prism 1.9.0.

/// The files of prism's Ruby library, by the feature name each is required as.
pub(crate) const PRISM_LIBRARIES: &[(&str, &str)] = &[
    ("prism", include_str!("prism/prism.rb")),
    ("prism/compiler", include_str!("prism/prism/compiler.rb")),
    (
        "prism/desugar_compiler",
        include_str!("prism/prism/desugar_compiler.rb"),
    ),
    (
        "prism/dispatcher",
        include_str!("prism/prism/dispatcher.rb"),
    ),
    (
        "prism/dot_visitor",
        include_str!("prism/prism/dot_visitor.rb"),
    ),
    ("prism/dsl", include_str!("prism/prism/dsl.rb")),
    (
        "prism/inspect_visitor",
        include_str!("prism/prism/inspect_visitor.rb"),
    ),
    (
        "prism/lex_compat",
        include_str!("prism/prism/lex_compat.rb"),
    ),
    (
        "prism/mutation_compiler",
        include_str!("prism/prism/mutation_compiler.rb"),
    ),
    ("prism/node", include_str!("prism/prism/node.rb")),
    ("prism/node_ext", include_str!("prism/prism/node_ext.rb")),
    ("prism/pack", include_str!("prism/prism/pack.rb")),
    (
        "prism/parse_result",
        include_str!("prism/prism/parse_result.rb"),
    ),
    (
        "prism/parse_result/comments",
        include_str!("prism/prism/parse_result/comments.rb"),
    ),
    (
        "prism/parse_result/errors",
        include_str!("prism/prism/parse_result/errors.rb"),
    ),
    (
        "prism/parse_result/newlines",
        include_str!("prism/prism/parse_result/newlines.rb"),
    ),
    ("prism/pattern", include_str!("prism/prism/pattern.rb")),
    (
        "prism/polyfill/append_as_bytes",
        include_str!("prism/prism/polyfill/append_as_bytes.rb"),
    ),
    (
        "prism/polyfill/byteindex",
        include_str!("prism/prism/polyfill/byteindex.rb"),
    ),
    (
        "prism/polyfill/scan_byte",
        include_str!("prism/prism/polyfill/scan_byte.rb"),
    ),
    (
        "prism/polyfill/unpack1",
        include_str!("prism/prism/polyfill/unpack1.rb"),
    ),
    (
        "prism/polyfill/warn",
        include_str!("prism/prism/polyfill/warn.rb"),
    ),
    (
        "prism/reflection",
        include_str!("prism/prism/reflection.rb"),
    ),
    (
        "prism/relocation",
        include_str!("prism/prism/relocation.rb"),
    ),
    ("prism/serialize", include_str!("prism/prism/serialize.rb")),
    (
        "prism/string_query",
        include_str!("prism/prism/string_query.rb"),
    ),
    (
        "prism/translation",
        include_str!("prism/prism/translation.rb"),
    ),
    (
        "prism/translation/parser",
        include_str!("prism/prism/translation/parser.rb"),
    ),
    (
        "prism/translation/parser/builder",
        include_str!("prism/prism/translation/parser/builder.rb"),
    ),
    (
        "prism/translation/parser/compiler",
        include_str!("prism/prism/translation/parser/compiler.rb"),
    ),
    (
        "prism/translation/parser/lexer",
        include_str!("prism/prism/translation/parser/lexer.rb"),
    ),
    (
        "prism/translation/parser_current",
        include_str!("prism/prism/translation/parser_current.rb"),
    ),
    (
        "prism/translation/parser_versions",
        include_str!("prism/prism/translation/parser_versions.rb"),
    ),
    (
        "prism/translation/ripper",
        include_str!("prism/prism/translation/ripper.rb"),
    ),
    (
        "prism/translation/ripper/filter",
        include_str!("prism/prism/translation/ripper/filter.rb"),
    ),
    (
        "prism/translation/ripper/lexer",
        include_str!("prism/prism/translation/ripper/lexer.rb"),
    ),
    (
        "prism/translation/ripper/sexp",
        include_str!("prism/prism/translation/ripper/sexp.rb"),
    ),
    (
        "prism/translation/ripper/shim",
        include_str!("prism/prism/translation/ripper/shim.rb"),
    ),
    (
        "prism/translation/ruby_parser",
        include_str!("prism/prism/translation/ruby_parser.rb"),
    ),
    ("prism/visitor", include_str!("prism/prism/visitor.rb")),
];
