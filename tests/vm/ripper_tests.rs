// Ripper: the scanner events and parser events a source produces, the
// lexer states it records, and the errors it reports.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

fn inspected(code: &str) -> String {
    let source = format!(
        "require 'ripper'\nanswer = begin\n{code}\nrescue StandardError => error\n  [error.class, error.message]\nend\nanswer.inspect"
    );
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let tokens = Lexer::new(&source).tokenize();
            let statements = Parser::new(tokens).parse().expect("parse failed");
            let mut vm = VirtualMachine::new();
            match vm.execute_program(&statements).expect("execution failed") {
                Some(Object::String(text)) => text.as_str().to_string(),
                other => panic!("expected an inspection, got {other:?}"),
            }
        })
        .expect("thread failed")
        .join()
        .expect("thread panicked")
}

#[test]
fn a_method_declaration_lexes_with_the_lexer_state_after_each_token() {
    assert_eq!(
        inspected("Ripper.lex('def m(a);nil end').map { |e| e[0...-1] + [e[-1].to_s] }"),
        "[[[1, 0], :on_kw, \"def\", \"FNAME\"], [[1, 3], :on_sp, \" \", \"FNAME\"], [[1, 4], :on_ident, \"m\", \"ENDFN\"], [[1, 5], :on_lparen, \"(\", \"BEG|LABEL\"], [[1, 6], :on_ident, \"a\", \"ARG\"], [[1, 7], :on_rparen, \")\", \"ENDFN\"], [[1, 8], :on_semicolon, \";\", \"BEG\"], [[1, 9], :on_kw, \"nil\", \"END\"], [[1, 12], :on_sp, \" \", \"END\"], [[1, 13], :on_kw, \"end\", \"END\"]]"
    );
}

#[test]
fn a_method_declaration_becomes_an_s_expression() {
    assert_eq!(
        inspected("Ripper.sexp('def hello; 42; end')"),
        "[:program, [[:def, [:@ident, \"hello\", [1, 4]], [:params, nil, nil, nil, nil, nil, nil, nil], [:bodystmt, [[:@int, \"42\", [1, 11]]], nil, nil, nil]]]]"
    );
}

#[test]
fn the_raw_s_expression_keeps_the_list_events() {
    assert_eq!(
        inspected("Ripper.sexp_raw('a, b = 1, 2')"),
        "[:program, [:stmts_add, [:stmts_new], [:massign, [:mlhs_add, [:mlhs_add, [:mlhs_new], [:var_field, [:@ident, \"a\", [1, 0]]]], [:var_field, [:@ident, \"b\", [1, 3]]]], [:mrhs_add, [:mrhs_new_from_args, [:args_add, [:args_new], [:@int, \"1\", [1, 7]]]], [:@int, \"2\", [1, 10]]]]]]"
    );
}

#[test]
fn a_local_variable_reads_as_a_variable_and_an_unknown_name_as_a_call() {
    assert_eq!(
        inspected("Ripper.sexp('x = 1; x; y')"),
        "[:program, [[:assign, [:var_field, [:@ident, \"x\", [1, 0]]], [:@int, \"1\", [1, 4]]], [:var_ref, [:@ident, \"x\", [1, 7]]], [:vcall, [:@ident, \"y\", [1, 10]]]]]"
    );
}

#[test]
fn a_known_local_changes_how_the_next_token_is_scanned() {
    assert_eq!(
        inspected("Ripper.lex(\"x = 1\\nx -1\").map { |e| [e[2], e[3].to_s] }.last(3)"),
        "[[\" \", \"END|LABEL\"], [\"-\", \"BEG\"], [\"1\", \"END\"]]"
    );
}

#[test]
fn a_syntax_error_answers_nil_from_sexp() {
    assert_eq!(inspected("Ripper.sexp('def (')"), "nil");
}

#[test]
fn a_syntax_error_raises_when_asked_to() {
    assert_eq!(
        inspected(
            "begin\n  Ripper.sexp('1 +', raise_errors: true)\nrescue SyntaxError => e\n  [e.class, e.message]\nend"
        ),
        "[SyntaxError, \"syntax error, unexpected end-of-input\"]"
    );
}

#[test]
fn a_jump_outside_any_loop_is_an_error() {
    assert_eq!(
        inspected("[Ripper.sexp('break'), Ripper.sexp('loop { break }').nil?]"),
        "[nil, false]"
    );
}

#[test]
fn a_jump_in_the_body_of_a_loop_modifier_is_accepted() {
    assert_eq!(
        inspected("Ripper.sexp('begin; break; end while true').nil?"),
        "false"
    );
}

#[test]
fn a_squiggly_heredoc_is_dedented() {
    assert_eq!(
        inspected("Ripper.sexp(\"<<~E\\n    a\\n      b\\nE\\n\")"),
        "[:program, [[:string_literal, [:string_content, [:@tstring_content, \"a\\n\", [2, 4]], [:@tstring_content, \"  b\\n\", [3, 4]]]]]]"
    );
}

#[test]
fn the_lexer_reports_the_indentation_it_dedented_as_ignored_space() {
    assert_eq!(
        inspected("Ripper.lex(\"<<~E\\n  a\\nE\\n\").map { |e| [e[1], e[2]] }"),
        "[[:on_heredoc_beg, \"<<~E\"], [:on_nl, \"\\n\"], [:on_ignored_sp, \"  \"], [:on_tstring_content, \"a\\n\"], [:on_heredoc_end, \"E\\n\"]]"
    );
}

#[test]
fn a_pattern_match_becomes_patterns() {
    assert_eq!(
        inspected("Ripper.sexp('x in [Integer => n, *rest]')"),
        "[:program, [[:case, [:vcall, [:@ident, \"x\", [1, 0]]], [:in, [:aryptn, nil, [[:binary, [:var_ref, [:@const, \"Integer\", [1, 6]]], :\"=>\", [:var_field, [:@ident, \"n\", [1, 17]]]]], [:var_field, [:@ident, \"rest\", [1, 21]]], nil], nil, nil]]]]"
    );
}

#[test]
fn a_binding_in_an_alternative_pattern_is_an_error() {
    assert_eq!(inspected("Ripper.sexp('case x; in [a] | [b]; end')"), "nil");
}

#[test]
fn a_command_with_a_do_block_takes_the_block() {
    assert_eq!(
        inspected("Ripper.sexp('f a do end')"),
        "[:program, [[:method_add_block, [:command, [:@ident, \"f\", [1, 0]], [:args_add_block, [[:vcall, [:@ident, \"a\", [1, 2]]]], false]], [:do_block, nil, [:bodystmt, [[:void_stmt]], nil, nil, nil]]]]]"
    );
}

#[test]
fn tokenize_answers_the_source_split_into_tokens() {
    assert_eq!(
        inspected("Ripper.tokenize(\"# note\\nputs 'hi'\")"),
        "[\"# note\\n\", \"puts\", \" \", \"'\", \"hi\", \"'\"]"
    );
}

#[test]
fn a_subclass_receives_the_events_it_defines() {
    assert_eq!(
        inspected(
            "class Idents < Ripper\n  def on_ident(token) = (@seen ||= []) << token\n  def seen = (parse; @seen)\nend\nIdents.new('alpha + beta').seen"
        ),
        "[\"alpha\", \"beta\"]"
    );
}

#[test]
fn a_filter_threads_a_value_through_the_tokens() {
    assert_eq!(
        inspected(
            "class Count < Ripper::Filter\n  def on_default(_event, _token, total) = total + 1\nend\nCount.new('a = 1').parse(0)"
        ),
        "5"
    );
}

#[test]
fn a_token_pattern_slices_the_matching_text() {
    assert_eq!(inspected("Ripper.slice('def m(a) end', 'ident')"), "\"m\"");
}

#[test]
fn an_unknown_token_name_in_a_pattern_is_refused() {
    assert_eq!(
        inspected("Ripper.slice('x', 'nonsense')"),
        "[Ripper::TokenPattern::CompileError, \"unknown token: nonsense\"]"
    );
}

#[test]
fn a_lexer_state_names_its_bits() {
    assert_eq!(
        inspected(
            "[Ripper.lex_state_name(Ripper::EXPR_BEG | Ripper::EXPR_LABEL), Ripper.lex_state_name(0)]"
        ),
        "[\"BEG|LABEL\", \"NONE\"]"
    );
}

#[test]
fn dedent_string_removes_columns_counting_a_tab_to_the_next_stop() {
    assert_eq!(
        inspected("text = +\"\\t x\"\n[Ripper.dedent_string(text, 9), text]"),
        "[2, \"x\"]"
    );
}

#[test]
fn a_magic_comment_is_reported() {
    assert_eq!(
        inspected(
            "class Magic < Ripper\n  def on_magic_comment(name, value) = (@found = [name, value])\n  attr_reader :found\nend\nreader = Magic.new(\"# frozen_string_literal: true\\nx\")\nreader.parse\nreader.found"
        ),
        "[\"frozen_string_literal\", \"true\"]"
    );
}

#[test]
fn the_source_after_end_is_not_scanned() {
    assert_eq!(
        inspected(
            "ripper = Ripper.new(\"1\\n__END__\\nignored\")\nripper.parse\n[ripper.end_seen?, Ripper.tokenize(\"1\\n__END__\\nignored\")]"
        ),
        "[true, [\"1\", \"\\n\", \"__END__\\n\"]]"
    );
}
