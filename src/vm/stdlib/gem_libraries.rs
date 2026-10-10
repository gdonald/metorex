// Written by scripts/vendor_gems.rb from irb 1.18.0, mutex_m 0.3.0, nkf 0.2.0, racc 1.8.1, reline 0.7.0, resolv-replace 0.1.1, rinda 0.2.0.

/// The files of the bundled gems metorex carries, by the feature name each
/// is required as.
pub(crate) const GEM_LIBRARIES: &[(&str, &str)] = &[
    ("irb", include_str!("gems/irb/irb.rb")),
    ("irb/cmd/nop", include_str!("gems/irb/irb/cmd/nop.rb")),
    ("irb/color", include_str!("gems/irb/irb/color.rb")),
    (
        "irb/color_printer",
        include_str!("gems/irb/irb/color_printer.rb"),
    ),
    ("irb/command", include_str!("gems/irb/irb/command.rb")),
    (
        "irb/command/backtrace",
        include_str!("gems/irb/irb/command/backtrace.rb"),
    ),
    (
        "irb/command/base",
        include_str!("gems/irb/irb/command/base.rb"),
    ),
    (
        "irb/command/break",
        include_str!("gems/irb/irb/command/break.rb"),
    ),
    (
        "irb/command/catch",
        include_str!("gems/irb/irb/command/catch.rb"),
    ),
    ("irb/command/cd", include_str!("gems/irb/irb/command/cd.rb")),
    (
        "irb/command/chws",
        include_str!("gems/irb/irb/command/chws.rb"),
    ),
    (
        "irb/command/context",
        include_str!("gems/irb/irb/command/context.rb"),
    ),
    (
        "irb/command/continue",
        include_str!("gems/irb/irb/command/continue.rb"),
    ),
    (
        "irb/command/copy",
        include_str!("gems/irb/irb/command/copy.rb"),
    ),
    (
        "irb/command/debug",
        include_str!("gems/irb/irb/command/debug.rb"),
    ),
    (
        "irb/command/delete",
        include_str!("gems/irb/irb/command/delete.rb"),
    ),
    (
        "irb/command/disable_irb",
        include_str!("gems/irb/irb/command/disable_irb.rb"),
    ),
    (
        "irb/command/edit",
        include_str!("gems/irb/irb/command/edit.rb"),
    ),
    (
        "irb/command/exit",
        include_str!("gems/irb/irb/command/exit.rb"),
    ),
    (
        "irb/command/finish",
        include_str!("gems/irb/irb/command/finish.rb"),
    ),
    (
        "irb/command/force_exit",
        include_str!("gems/irb/irb/command/force_exit.rb"),
    ),
    (
        "irb/command/help",
        include_str!("gems/irb/irb/command/help.rb"),
    ),
    (
        "irb/command/history",
        include_str!("gems/irb/irb/command/history.rb"),
    ),
    (
        "irb/command/info",
        include_str!("gems/irb/irb/command/info.rb"),
    ),
    (
        "irb/command/internal_helpers",
        include_str!("gems/irb/irb/command/internal_helpers.rb"),
    ),
    (
        "irb/command/irb_info",
        include_str!("gems/irb/irb/command/irb_info.rb"),
    ),
    (
        "irb/command/load",
        include_str!("gems/irb/irb/command/load.rb"),
    ),
    ("irb/command/ls", include_str!("gems/irb/irb/command/ls.rb")),
    (
        "irb/command/measure",
        include_str!("gems/irb/irb/command/measure.rb"),
    ),
    (
        "irb/command/next",
        include_str!("gems/irb/irb/command/next.rb"),
    ),
    (
        "irb/command/pushws",
        include_str!("gems/irb/irb/command/pushws.rb"),
    ),
    (
        "irb/command/show_doc",
        include_str!("gems/irb/irb/command/show_doc.rb"),
    ),
    (
        "irb/command/show_source",
        include_str!("gems/irb/irb/command/show_source.rb"),
    ),
    (
        "irb/command/step",
        include_str!("gems/irb/irb/command/step.rb"),
    ),
    (
        "irb/command/subirb",
        include_str!("gems/irb/irb/command/subirb.rb"),
    ),
    (
        "irb/command/whereami",
        include_str!("gems/irb/irb/command/whereami.rb"),
    ),
    ("irb/completion", include_str!("gems/irb/irb/completion.rb")),
    ("irb/context", include_str!("gems/irb/irb/context.rb")),
    ("irb/debug", include_str!("gems/irb/irb/debug.rb")),
    ("irb/debug/ui", include_str!("gems/irb/irb/debug/ui.rb")),
    (
        "irb/default_commands",
        include_str!("gems/irb/irb/default_commands.rb"),
    ),
    ("irb/easter-egg", include_str!("gems/irb/irb/easter-egg.rb")),
    (
        "irb/ext/change-ws",
        include_str!("gems/irb/irb/ext/change-ws.rb"),
    ),
    (
        "irb/ext/eval_history",
        include_str!("gems/irb/irb/ext/eval_history.rb"),
    ),
    ("irb/ext/loader", include_str!("gems/irb/irb/ext/loader.rb")),
    (
        "irb/ext/multi-irb",
        include_str!("gems/irb/irb/ext/multi-irb.rb"),
    ),
    ("irb/ext/tracer", include_str!("gems/irb/irb/ext/tracer.rb")),
    (
        "irb/ext/use-loader",
        include_str!("gems/irb/irb/ext/use-loader.rb"),
    ),
    (
        "irb/ext/workspaces",
        include_str!("gems/irb/irb/ext/workspaces.rb"),
    ),
    ("irb/frame", include_str!("gems/irb/irb/frame.rb")),
    ("irb/help", include_str!("gems/irb/irb/help.rb")),
    (
        "irb/helper_method",
        include_str!("gems/irb/irb/helper_method.rb"),
    ),
    (
        "irb/helper_method/base",
        include_str!("gems/irb/irb/helper_method/base.rb"),
    ),
    (
        "irb/helper_method/conf",
        include_str!("gems/irb/irb/helper_method/conf.rb"),
    ),
    ("irb/history", include_str!("gems/irb/irb/history.rb")),
    ("irb/init", include_str!("gems/irb/irb/init.rb")),
    (
        "irb/input-method",
        include_str!("gems/irb/irb/input-method.rb"),
    ),
    ("irb/inspector", include_str!("gems/irb/irb/inspector.rb")),
    ("irb/lc/error", include_str!("gems/irb/irb/lc/error.rb")),
    (
        "irb/lc/ja/error",
        include_str!("gems/irb/irb/lc/ja/error.rb"),
    ),
    ("irb/locale", include_str!("gems/irb/irb/locale.rb")),
    (
        "irb/nesting_parser",
        include_str!("gems/irb/irb/nesting_parser.rb"),
    ),
    ("irb/notifier", include_str!("gems/irb/irb/notifier.rb")),
    (
        "irb/output-method",
        include_str!("gems/irb/irb/output-method.rb"),
    ),
    ("irb/pager", include_str!("gems/irb/irb/pager.rb")),
    ("irb/ruby-lex", include_str!("gems/irb/irb/ruby-lex.rb")),
    (
        "irb/source_finder",
        include_str!("gems/irb/irb/source_finder.rb"),
    ),
    (
        "irb/startup_message",
        include_str!("gems/irb/irb/startup_message.rb"),
    ),
    ("irb/statement", include_str!("gems/irb/irb/statement.rb")),
    ("irb/version", include_str!("gems/irb/irb/version.rb")),
    ("irb/workspace", include_str!("gems/irb/irb/workspace.rb")),
    (
        "irb/ws-for-case-2",
        include_str!("gems/irb/irb/ws-for-case-2.rb"),
    ),
    ("irb/xmp", include_str!("gems/irb/irb/xmp.rb")),
    ("mutex_m", include_str!("gems/mutex_m/mutex_m.rb")),
    ("kconv", include_str!("gems/nkf/kconv.rb")),
    ("nkf", include_str!("gems/nkf/nkf.rb")),
    ("racc", include_str!("gems/racc/racc.rb")),
    ("racc/compat", include_str!("gems/racc/racc/compat.rb")),
    (
        "racc/debugflags",
        include_str!("gems/racc/racc/debugflags.rb"),
    ),
    (
        "racc/exception",
        include_str!("gems/racc/racc/exception.rb"),
    ),
    ("racc/grammar", include_str!("gems/racc/racc/grammar.rb")),
    (
        "racc/grammarfileparser",
        include_str!("gems/racc/racc/grammarfileparser.rb"),
    ),
    ("racc/info", include_str!("gems/racc/racc/info.rb")),
    ("racc/iset", include_str!("gems/racc/racc/iset.rb")),
    (
        "racc/logfilegenerator",
        include_str!("gems/racc/racc/logfilegenerator.rb"),
    ),
    (
        "racc/parser-text",
        include_str!("gems/racc/racc/parser-text.rb"),
    ),
    ("racc/parser", include_str!("gems/racc/racc/parser.rb")),
    (
        "racc/parserfilegenerator",
        include_str!("gems/racc/racc/parserfilegenerator.rb"),
    ),
    (
        "racc/sourcetext",
        include_str!("gems/racc/racc/sourcetext.rb"),
    ),
    ("racc/state", include_str!("gems/racc/racc/state.rb")),
    (
        "racc/statetransitiontable",
        include_str!("gems/racc/racc/statetransitiontable.rb"),
    ),
    ("racc/static", include_str!("gems/racc/racc/static.rb")),
    ("reline", include_str!("gems/reline/reline.rb")),
    (
        "reline/config",
        include_str!("gems/reline/reline/config.rb"),
    ),
    ("reline/face", include_str!("gems/reline/reline/face.rb")),
    (
        "reline/history",
        include_str!("gems/reline/reline/history.rb"),
    ),
    ("reline/io", include_str!("gems/reline/reline/io.rb")),
    (
        "reline/io/ansi",
        include_str!("gems/reline/reline/io/ansi.rb"),
    ),
    (
        "reline/io/dumb",
        include_str!("gems/reline/reline/io/dumb.rb"),
    ),
    (
        "reline/io/windows",
        include_str!("gems/reline/reline/io/windows.rb"),
    ),
    (
        "reline/key_actor",
        include_str!("gems/reline/reline/key_actor.rb"),
    ),
    (
        "reline/key_actor/base",
        include_str!("gems/reline/reline/key_actor/base.rb"),
    ),
    (
        "reline/key_actor/composite",
        include_str!("gems/reline/reline/key_actor/composite.rb"),
    ),
    (
        "reline/key_actor/emacs",
        include_str!("gems/reline/reline/key_actor/emacs.rb"),
    ),
    (
        "reline/key_actor/vi_command",
        include_str!("gems/reline/reline/key_actor/vi_command.rb"),
    ),
    (
        "reline/key_actor/vi_insert",
        include_str!("gems/reline/reline/key_actor/vi_insert.rb"),
    ),
    (
        "reline/key_stroke",
        include_str!("gems/reline/reline/key_stroke.rb"),
    ),
    (
        "reline/kill_ring",
        include_str!("gems/reline/reline/kill_ring.rb"),
    ),
    (
        "reline/line_editor",
        include_str!("gems/reline/reline/line_editor.rb"),
    ),
    (
        "reline/unicode",
        include_str!("gems/reline/reline/unicode.rb"),
    ),
    (
        "reline/unicode/east_asian_width",
        include_str!("gems/reline/reline/unicode/east_asian_width.rb"),
    ),
    (
        "reline/version",
        include_str!("gems/reline/reline/version.rb"),
    ),
    (
        "resolv-replace",
        include_str!("gems/resolv-replace/resolv-replace.rb"),
    ),
    ("rinda/rinda", include_str!("gems/rinda/rinda/rinda.rb")),
    ("rinda/ring", include_str!("gems/rinda/rinda/ring.rb")),
    (
        "rinda/tuplespace",
        include_str!("gems/rinda/rinda/tuplespace.rb"),
    ),
];
