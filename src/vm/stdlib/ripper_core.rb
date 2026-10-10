# Ripper: Ruby's parser, exposed as a stream of events. Every token the
# scanner reads and every rule the grammar reduces calls an `on_` method,
# and what that method returns is what the enclosing rule receives.

class Ripper
  PARSER_EVENT_TABLE = {
    BEGIN: 1, END: 1, alias: 2, alias_error: 2, aref: 2, aref_field: 2,
    arg_ambiguous: 1, arg_paren: 1, args_add: 2, args_add_block: 2, args_add_star: 2, args_forward: 0,
    args_new: 0, array: 1, aryptn: 4, assign: 2, assign_error: 2, assoc_new: 2,
    assoc_splat: 1, assoclist_from_args: 1, bare_assoc_hash: 1, begin: 1, binary: 3, block_var: 2,
    blockarg: 1, bodystmt: 4, brace_block: 2, break: 1, call: 3, case: 2,
    class: 3, class_name_error: 2, command: 2, command_call: 4, const_path_field: 2, const_path_ref: 2,
    const_ref: 1, def: 3, defined: 1, defs: 5, do_block: 2, dot2: 2,
    dot3: 2, dyna_symbol: 1, else: 1, elsif: 3, ensure: 1, excessed_comma: 0,
    fcall: 1, field: 3, fndptn: 4, for: 3, hash: 1, heredoc_dedent: 2,
    hshptn: 3, if: 3, if_mod: 2, ifop: 3, in: 3, kwrest_param: 1,
    lambda: 2, magic_comment: 2, massign: 2, method_add_arg: 2, method_add_block: 2, mlhs_add: 2,
    mlhs_add_post: 2, mlhs_add_star: 2, mlhs_new: 0, mlhs_paren: 1, module: 2, mrhs_add: 2,
    mrhs_add_star: 2, mrhs_new: 0, mrhs_new_from_args: 1, next: 1, nokw_param: 1, opassign: 3,
    operator_ambiguous: 2, param_error: 2, params: 7, paren: 1, parse_error: 1, program: 1,
    qsymbols_add: 2, qsymbols_new: 0, qwords_add: 2, qwords_new: 0, redo: 0, regexp_add: 2,
    regexp_literal: 2, regexp_new: 0, rescue: 4, rescue_mod: 2, rest_param: 1, retry: 0,
    return: 1, return0: 0, sclass: 2, stmts_add: 2, stmts_new: 0, string_add: 2,
    string_concat: 2, string_content: 0, string_dvar: 1, string_embexpr: 1, string_literal: 1, super: 1,
    symbol: 1, symbol_literal: 1, symbols_add: 2, symbols_new: 0, top_const_field: 1, top_const_ref: 1,
    unary: 2, undef: 1, unless: 3, unless_mod: 2, until: 2, until_mod: 2,
    var_alias: 2, var_field: 1, var_ref: 1, vcall: 1, void_stmt: 0, when: 3,
    while: 2, while_mod: 2, word_add: 2, word_new: 0, words_add: 2, words_new: 0,
    xstring_add: 2, xstring_literal: 1, xstring_new: 0, yield: 1, yield0: 0, zsuper: 0
  }.freeze
  SCANNER_EVENT_TABLE = {
    CHAR: 1, __end__: 1, backref: 1, backtick: 1, comma: 1, comment: 1,
    const: 1, cvar: 1, embdoc: 1, embdoc_beg: 1, embdoc_end: 1, embexpr_beg: 1,
    embexpr_end: 1, embvar: 1, float: 1, gvar: 1, heredoc_beg: 1, heredoc_end: 1,
    ident: 1, ignored_nl: 1, imaginary: 1, int: 1, ivar: 1, kw: 1,
    label: 1, label_end: 1, lbrace: 1, lbracket: 1, lparen: 1, nl: 1,
    op: 1, period: 1, qsymbols_beg: 1, qwords_beg: 1, rational: 1, rbrace: 1,
    rbracket: 1, regexp_beg: 1, regexp_end: 1, rparen: 1, semicolon: 1, sp: 1,
    symbeg: 1, symbols_beg: 1, tlambda: 1, tlambeg: 1, tstring_beg: 1, tstring_content: 1,
    tstring_end: 1, words_beg: 1, words_sep: 1, ignored_sp: 1
  }

  PARSER_EVENTS = PARSER_EVENT_TABLE.keys
  SCANNER_EVENTS = SCANNER_EVENT_TABLE.keys
  EVENTS = PARSER_EVENTS + SCANNER_EVENTS

  Version = "0.1.0"

  EXPR_BEG = 1
  EXPR_END = 2
  EXPR_ENDARG = 4
  EXPR_ENDFN = 8
  EXPR_ARG = 16
  EXPR_CMDARG = 32
  EXPR_MID = 64
  EXPR_FNAME = 128
  EXPR_DOT = 256
  EXPR_CLASS = 512
  EXPR_LABEL = 1024
  EXPR_LABELED = 2048
  EXPR_FITEM = 4096
  EXPR_VALUE = EXPR_BEG
  EXPR_BEG_ANY = EXPR_BEG | EXPR_MID | EXPR_CLASS
  EXPR_ARG_ANY = EXPR_ARG | EXPR_CMDARG
  EXPR_END_ANY = EXPR_END | EXPR_ENDARG | EXPR_ENDFN
  EXPR_NONE = 0

  LEX_STATE_NAMES = %w[BEG END ENDARG ENDFN ARG CMDARG MID FNAME DOT CLASS LABEL LABELED FITEM].freeze
  private_constant :LEX_STATE_NAMES

  # The names of the bits set in a lexer state, joined by `|`.
  def self.lex_state_name(state)
    names = LEX_STATE_NAMES.each_with_index.filter_map { |name, bit| name if state.anybits?(1 << bit) }
    names.empty? ? "NONE" : names.join("|")
  end

  # Removes up to `width` columns of leading blanks from `input` in place,
  # a tab reaching the next multiple of eight, and returns how many bytes
  # it removed.
  def self.dedent_string(input, width)
    raise TypeError, "no implicit conversion of #{input.class} into String" unless input.is_a?(String)

    column = 0
    removed = 0
    input.each_byte do |byte|
      break if column >= width

      if byte == 32
        column += 1
      elsif byte == 9
        reached = (column / 8 + 1) * 8
        break if reached > width

        column = reached
      else
        break
      end
      removed += 1
    end
    input.replace(input.byteslice(removed..)) if removed.positive?
    removed
  end

  def self.parse(src, filename = "(ripper)", lineno = 1)
    new(src, filename, lineno).parse
  end

  def initialize(src, filename = "(ripper)", lineno = 1)
    source = if src.is_a?(String)
               src
             elsif src.respond_to?(:gets)
               lines = +""
               while (line = src.gets)
                 lines << line
               end
               lines
             elsif src.respond_to?(:to_str)
               src.to_str
             else
               raise TypeError, "wrong argument type #{src.class} (expected String or IO)"
             end
    @__ripper_source = source
    @__ripper_filename = filename.nil? ? nil : filename.to_str
    @__ripper_start_line = lineno.to_int
    @__ripper_lineno = nil
    @__ripper_column = nil
    @__ripper_state = nil
    @__ripper_token = nil
    @__ripper_error = false
    @__ripper_end_seen = false
    @__ripper_parsed = false
    @__ripper_yydebug = false
    @__ripper_debug_output = $stdout
  end

  def parse
    raise ArgumentError, "method called for uninitialized object" unless defined?(@__ripper_source)
    return nil if @__ripper_parsed

    @__ripper_parsed = true
    bridge = Engine::Bridge.new(self)
    scanner = Engine::Scanner.new(bridge, @__ripper_source, @__ripper_start_line)
    grammar = Engine::Grammar.new(bridge, scanner)
    bridge.grammar = grammar
    result = grammar.run
    @__ripper_end_seen = scanner.end_seen
    result
  end

  def lineno = @__ripper_lineno
  def column = @__ripper_column
  def filename = @__ripper_filename
  def state = @__ripper_state
  def token = @__ripper_token
  def encoding = @__ripper_source.encoding
  def end_seen? = @__ripper_end_seen
  def error? = @__ripper_error
  def yydebug = @__ripper_yydebug

  def yydebug=(flag)
    @__ripper_yydebug = flag
  end

  def debug_output = @__ripper_debug_output

  def debug_output=(output)
    @__ripper_debug_output = output
  end

  private

  def dedent_string(input, width) = Ripper.dedent_string(input, width)

  def _dispatch_0 = nil
  def _dispatch_1(a) = a
  def _dispatch_2(a, _b) = a
  def _dispatch_3(a, _b, _c) = a
  def _dispatch_4(a, _b, _c, _d) = a
  def _dispatch_5(a, _b, _c, _d, _e) = a
  def _dispatch_6(a, _b, _c, _d, _e, _f) = a
  def _dispatch_7(a, _b, _c, _d, _e, _f, _g) = a

  PARSER_EVENT_TABLE.each do |id, arity|
    alias_method "on_#{id}", "_dispatch_#{arity}"
  end

  SCANNER_EVENTS.each do |id|
    alias_method "on_#{id}", :_dispatch_1
  end

  def warn(fmt, *args); end

  def warning(fmt, *args); end

  def compile_error(msg); end

  module Engine
    # Carries the scanner's and the grammar's events to the ripper's
    # `on_` methods.
    class Bridge
      MAGIC_COMMENTS = %w[coding encoding frozen_string_literal warn_indent shareable_constant_value
                          warn_past_scope].freeze

      attr_writer :grammar

      def initialize(ripper)
        @ripper = ripper
      end

      def scan_event(event, text, line, column, state)
        @ripper.instance_variable_set(:@__ripper_lineno, line)
        @ripper.instance_variable_set(:@__ripper_column, column)
        @ripper.instance_variable_set(:@__ripper_token, text)
        @ripper.instance_variable_set(:@__ripper_state, state)
        @ripper.__send__(event, text)
      end

      def parser_event(name, *args)
        @ripper.__send__(:"on_#{name}", *args)
      end

      def parse_error(message, token = nil)
        if token
          @ripper.instance_variable_set(:@__ripper_lineno, token.line)
          @ripper.instance_variable_set(:@__ripper_column, token.column)
          @ripper.instance_variable_set(:@__ripper_token, token.text)
        end
        @ripper.instance_variable_set(:@__ripper_error, true)
        @ripper.__send__(:on_parse_error, message)
      end

      def compile_error(message)
        @ripper.instance_variable_set(:@__ripper_error, true)
        @ripper.__send__(:compile_error, message)
      end

      # An error the grammar found in a program it can still read, through
      # the event MRI's parser reports it with: `parse_error`, or one of
      # `assign_error`, `alias_error` and `class_name_error`, which are also
      # handed the token the error names.
      def error_event(event, message, token)
        return compile_error(message) if event == :compile_error

        @ripper.instance_variable_set(:@__ripper_error, true)
        return @ripper.__send__(:on_parse_error, message) if event == :parse_error

        @ripper.__send__(:"on_#{event}", message, token&.value)
      end

      def local?(name) = @grammar ? @grammar.local?(name) : false

      # An error the scanner reports and reads on past, at a place on a
      # line rather than at a token.
      def scanner_error(message, line, from, to, state)
        @ripper.instance_variable_set(:@__ripper_lineno, line)
        @ripper.instance_variable_set(:@__ripper_column, from)
        @ripper.instance_variable_set(:@__ripper_state, state)
        return @grammar.report_scanned_error(message, line, from, to) if @grammar

        error_event(Grammar.error_event_for(message), message, nil)
      end

      # A warning MRI's parser gives while reading, at the token it names.
      def warn(message, token = nil)
        if token
          @ripper.instance_variable_set(:@__ripper_lineno, token.line)
          @ripper.instance_variable_set(:@__ripper_column, token.column)
        end
        @ripper.__send__(:warn, message)
      end

      # The file name errors give, which a Ripper is made with.
      def filename = @ripper.filename

      def magic_comment(comment)
        body = comment.sub(/\A#\s*/, "").chomp
        pairs = if body =~ /-\*-(.*)-\*-/
                  Regexp.last_match(1).split(";").map { |pair| pair.split(":", 2).map(&:strip) }
                elsif body =~ /\A([\w-]+)\s*:\s*(\S+)/
                  [[Regexp.last_match(1), Regexp.last_match(2)]]
                else
                  []
                end
        coding = nil
        pairs.each do |name, value|
          key = name.tr("-", "_").downcase
          next unless value && MAGIC_COMMENTS.include?(key)

          coding = value if %w[coding encoding].include?(key)
          parser_event(:magic_comment, name, value)
        end
        coding
      end
    end
  end
end
