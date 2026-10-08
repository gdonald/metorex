# Coloring Ruby source for a terminal. The tokens come from Ripper, each named
# by the token type MRI's Prism lexer gives it, and a walk over Ripper's tree
# marks the method names, calls and symbols the token types alone do not
# tell apart.
require "ripper"

module IRB # :nodoc:
  module Color
    CLEAR     = 0
    BOLD      = 1
    UNDERLINE = 4
    REVERSE   = 7
    BLACK     = 30
    RED       = 31
    GREEN     = 32
    YELLOW    = 33
    BLUE      = 34
    MAGENTA   = 35
    CYAN      = 36
    WHITE     = 37

    TOKEN_SEQS = {
      KEYWORD_NIL:        [CYAN, BOLD],
      KEYWORD_SELF:       [CYAN, BOLD],
      KEYWORD_TRUE:       [CYAN, BOLD],
      KEYWORD_FALSE:      [CYAN, BOLD],
      KEYWORD___FILE__:   [CYAN, BOLD],
      KEYWORD___LINE__:   [CYAN, BOLD],
      KEYWORD___ENCODING__: [CYAN, BOLD],
      CHARACTER_LITERAL:  [BLUE, BOLD],
      BACK_REFERENCE:     [GREEN, BOLD],
      BACKTICK:           [RED, BOLD],
      COMMENT:            [BLUE, BOLD],
      EMBDOC_BEGIN:       [BLUE, BOLD],
      EMBDOC_LINE:        [BLUE, BOLD],
      EMBDOC_END:         [BLUE, BOLD],
      CONSTANT:           [BLUE, BOLD, UNDERLINE],
      EMBEXPR_BEGIN:      [RED],
      EMBEXPR_END:        [RED],
      EMBVAR:             [RED],
      FLOAT:              [MAGENTA, BOLD],
      GLOBAL_VARIABLE:    [GREEN, BOLD],
      HEREDOC_START:      [RED],
      HEREDOC_END:        [RED],
      FLOAT_IMAGINARY:    [BLUE, BOLD],
      INTEGER_IMAGINARY:  [BLUE, BOLD],
      FLOAT_RATIONAL_IMAGINARY:  [BLUE, BOLD],
      INTEGER_RATIONAL_IMAGINARY:  [BLUE, BOLD],
      INTEGER:            [BLUE, BOLD],
      INTEGER_RATIONAL:   [BLUE, BOLD],
      FLOAT_RATIONAL:     [BLUE, BOLD],
      KEYWORD_END:        [GREEN],
      KEYWORD_CLASS:      [GREEN],
      KEYWORD_MODULE:     [GREEN],
      KEYWORD_IF:         [GREEN],
      KEYWORD_IF_MODIFIER: [GREEN],
      KEYWORD_UNLESS_MODIFIER: [GREEN],
      KEYWORD_WHILE_MODIFIER: [GREEN],
      KEYWORD_UNTIL_MODIFIER: [GREEN],
      KEYWORD_RESCUE_MODIFIER: [GREEN],
      KEYWORD_THEN:       [GREEN],
      KEYWORD_UNLESS:     [GREEN],
      KEYWORD_ELSE:       [GREEN],
      KEYWORD_ELSIF:      [GREEN],
      KEYWORD_WHILE:      [GREEN],
      KEYWORD_UNTIL:      [GREEN],
      KEYWORD_CASE:       [GREEN],
      KEYWORD_WHEN:       [GREEN],
      KEYWORD_IN:         [GREEN],
      KEYWORD_DEF:        [GREEN],
      KEYWORD_DO:         [GREEN],
      KEYWORD_DO_BLOCK:   [GREEN],
      KEYWORD_DO_LOOP:    [GREEN],
      KEYWORD_FOR:        [GREEN],
      KEYWORD_BEGIN:      [GREEN],
      KEYWORD_RESCUE:     [GREEN],
      KEYWORD_ENSURE:     [GREEN],
      KEYWORD_ALIAS:      [GREEN],
      KEYWORD_UNDEF:      [GREEN],
      KEYWORD_BEGIN_UPCASE: [GREEN],
      KEYWORD_END_UPCASE: [GREEN],
      KEYWORD_YIELD:      [GREEN],
      KEYWORD_REDO:       [GREEN],
      KEYWORD_RETRY:      [GREEN],
      KEYWORD_NEXT:       [GREEN],
      KEYWORD_BREAK:      [GREEN],
      KEYWORD_SUPER:      [GREEN],
      KEYWORD_RETURN:     [GREEN],
      KEYWORD_DEFINED:    [GREEN],
      KEYWORD_NOT:        [GREEN],
      KEYWORD_AND:        [GREEN],
      KEYWORD_OR:         [GREEN],
      LABEL:              [MAGENTA],
      LABEL_END:          [RED, BOLD],
      NUMBERED_REFERENCE: [GREEN, BOLD],
      PERCENT_UPPER_W:    [RED, BOLD],
      PERCENT_LOWER_W:    [RED, BOLD],
      PERCENT_LOWER_X:    [RED, BOLD],
      REGEXP_BEGIN:       [RED, BOLD],
      REGEXP_END:         [RED, BOLD],
      STRING_BEGIN:       [RED, BOLD],
      STRING_CONTENT:     [RED],
      STRING_END:         [RED, BOLD],
      __END__:            [GREEN],
      method_name:        [CYAN, BOLD],
      message_name:       [CYAN],
      symbol:             [YELLOW],
      error:              [RED, REVERSE],
    }.transform_values do |styles|
      styles.map { |style| "\e[#{style}m" }.join
    end
    CLEAR_SEQ = "\e[#{CLEAR}m"
    OPERATORS = %i(!= !~ =~ == === <=> > >= < <= & | ^ >> << - + % / * ** -@ +@ ~ ! [] []=)
    private_constant :TOKEN_SEQS, :CLEAR_SEQ, :OPERATORS

    # The token type Prism gives a Ripper event that names one type whatever
    # its text.
    EVENT_TYPES = {
      on_comment: :COMMENT,
      on_embdoc_beg: :EMBDOC_BEGIN,
      on_embdoc: :EMBDOC_LINE,
      on_embdoc_end: :EMBDOC_END,
      on_const: :CONSTANT,
      on_embexpr_beg: :EMBEXPR_BEGIN,
      on_embexpr_end: :EMBEXPR_END,
      on_embvar: :EMBVAR,
      on_float: :FLOAT,
      on_int: :INTEGER,
      on_gvar: :GLOBAL_VARIABLE,
      on_backref: :BACK_REFERENCE,
      on_nth_ref: :NUMBERED_REFERENCE,
      on_heredoc_beg: :HEREDOC_START,
      on_heredoc_end: :HEREDOC_END,
      on_CHAR: :CHARACTER_LITERAL,
      on_label: :LABEL,
      on_label_end: :LABEL_END,
      on_qwords_beg: :PERCENT_LOWER_W,
      on_words_beg: :PERCENT_UPPER_W,
      on_regexp_beg: :REGEXP_BEGIN,
      on_regexp_end: :REGEXP_END,
      on_tstring_beg: :STRING_BEGIN,
      on_tstring_content: :STRING_CONTENT,
      on_tstring_end: :STRING_END,
      on___end__: :__END__,
      on_ident: :IDENTIFIER,
      on_ivar: :INSTANCE_VARIABLE,
      on_cvar: :CLASS_VARIABLE,
      on_op: :OPERATOR,
      on_nl: :NEWLINE,
      on_ignored_nl: :IGNORED_NEWLINE,
      on_words_sep: :WORDS_SEP,
    }.freeze

    # The keywords whose token type names the value they stand for.
    VALUE_KEYWORDS = {
      "nil" => :KEYWORD_NIL,
      "self" => :KEYWORD_SELF,
      "true" => :KEYWORD_TRUE,
      "false" => :KEYWORD_FALSE,
      "__FILE__" => :KEYWORD___FILE__,
      "__LINE__" => :KEYWORD___LINE__,
      "__ENCODING__" => :KEYWORD___ENCODING__,
    }.freeze

    # Control characters as a terminal shows them typed.
    ESCAPED_PAIRS = ((0x00..0x1F).to_h { |ord| [ord, "^#{(ord + 64).chr}"] }).merge(0x7F => "^?").freeze
    BYTE_ORDER_MARK = "\xEF\xBB\xBF".b.freeze
    private_constant :EVENT_TYPES, :VALUE_KEYWORDS, :ESCAPED_PAIRS, :BYTE_ORDER_MARK

    class << self
      def colorable?
        supported = $stdout.tty? && (/mswin|mingw/.match?(RUBY_PLATFORM) || (ENV.key?("TERM") && ENV["TERM"] != "dumb"))
        if IRB.respond_to?(:conf)
          supported && !!IRB.conf.fetch(:USE_COLORIZE, true)
        else
          supported
        end
      end

      def inspect_colorable?(obj, seen: {}.compare_by_identity)
        case obj
        when String, Symbol, Regexp, Integer, Float, FalseClass, TrueClass, NilClass
          true
        when Hash
          without_circular_ref(obj, seen: seen) do
            obj.all? { |k, v| inspect_colorable?(k, seen: seen) && inspect_colorable?(v, seen: seen) }
          end
        when Array
          without_circular_ref(obj, seen: seen) do
            obj.all? { |o| inspect_colorable?(o, seen: seen) }
          end
        when Range
          inspect_colorable?(obj.begin, seen: seen) && inspect_colorable?(obj.end, seen: seen)
        when Module
          !obj.name.nil?
        else
          false
        end
      end

      def clear(colorable: colorable?)
        colorable ? CLEAR_SEQ : ""
      end

      def colorize(text, seq, colorable: colorable?)
        return text unless colorable
        seq = seq.map { |s| "\e[#{const_get(s)}m" }.join("")
        "#{seq}#{text}#{CLEAR_SEQ}"
      end

      def colorize_code(code, complete: true, ignore_error: false, colorable: colorable?, local_variables: [])
        return code unless colorable

        # Ripper counts columns from after a byte order mark.
        if code.b.start_with?(BYTE_ORDER_MARK)
          rest = colorize_code(code.byteslice(BYTE_ORDER_MARK.bytesize..), complete: complete, ignore_error: ignore_error, colorable: colorable, local_variables: local_variables)
          return code.byteslice(0, BYTE_ORDER_MARK.bytesize) + rest
        end

        tree = Ripper.sexp(code)
        return escape_for_print(code) if ignore_error && tree.nil?

        lexed = joined_string_content(Ripper.lex(code).reject { |_, event,| event == :on_sp })
        spans = TreeSpans.new(local_variables)
        spans.walk(tree) if tree
        tokens = lexed.map do |(line, column), event, text|
          span(line, column, 2, token_type(event, text), text)
        end
        spans.tokens.concat(symbol_spans(lexed))

        colored = +""
        line_index = 0
        col = 0
        lines = code.lines
        flush = lambda do |next_line_index, next_col|
          return if next_line_index == line_index && next_col == col
          (line_index...[next_line_index, lines.size].min).each do |ln|
            colored << escape_for_print(lines[line_index].byteslice(col..))
            line_index = ln + 1
            col = 0
          end
          unless col == next_col
            colored << escape_for_print(lines[next_line_index].byteslice(col..next_col - 1))
          end
        end

        (spans.tokens + tokens).sort.each do |start_line, start_column, _priority, end_line, end_column, type, value|
          next if start_line - 1 < line_index || (start_line - 1 == line_index && start_column < col)

          flush.call(start_line - 1, start_column)
          color = TOKEN_SEQS[type]
          if type == :__END__
            end_line = start_line
            value = "__END__"
            end_column = start_column + 7
          end
          if color
            value.split(/(\n)/).each do |s|
              colored << (s == "\n" ? s : "#{color}#{escape_for_print(s)}#{CLEAR_SEQ}")
            end
          else
            colored << value
          end
          line_index = end_line - 1
          col = end_column
        end
        flush.call lines.size, 0
        colored
      end

      private

      # A token as the merge reads it: where it starts, which of two at the
      # same place wins, where it ends, its type and its text.
      def span(line, column, priority, type, text)
        breaks = text.count("\n")
        end_column = breaks.zero? ? column + text.bytesize : text.bytesize - text.b.rindex("\n") - 1
        [line, column, priority, line + breaks, end_column, type, text]
      end

      class TreeSpans
        attr_reader :tokens

        def initialize(local_variables)
          @tokens = []
          @local_variables = local_variables.map(&:to_s)
        end

        # `it_names_parameter` holds inside a block written without
        # parameters, where `it` reads the block's argument rather than
        # calling a method. `target` holds for what a multiple assignment,
        # `for` or `rescue =>` assigns to, where `recv.name` is a target
        # Prism does not color as a call.
        def walk(node, it_names_parameter: false, target: false)
          return unless node.is_a?(Array)
          case node.first
          when :def, :defs, :class, :sclass, :module
            it_names_parameter = false
            method_name node[1] if node.first == :def
            method_name node[3] if node.first == :defs
          when :brace_block, :do_block
            it_names_parameter = node[1].nil?
          when :lambda
            it_names_parameter = node[1].is_a?(Array) && node[1].first == :params && node[1].drop(1).all?(&:nil?)
          when :alias
            node[1..2].each { |name| method_name(name[1]) if bare_name?(name) }
          when :undef
            node[1].each { |name| symbol(name[1]) if bare_name?(name) }
          when :command, :fcall
            message node[1], nil
          when :vcall
            unless @local_variables.include?(node[1][1]) || (it_names_parameter && node[1][1] == "it")
              message node[1], nil
            end
          when :call, :command_call
            message node[3], node[2]
          when :field
            message node[3], node[2] unless target
          when :massign
            node[1].each { |held| walk(held, it_names_parameter: it_names_parameter, target: true) }
            return walk(node[2], it_names_parameter: it_names_parameter)
          when :mlhs, :rest_param
            return node.each { |held| walk(held, it_names_parameter: it_names_parameter, target: true) }
          when :for
            walk(node[1], it_names_parameter: it_names_parameter, target: true)
            return node.drop(2).each { |held| walk(held, it_names_parameter: it_names_parameter) }
          when :rescue
            walk(node[2], it_names_parameter: it_names_parameter, target: true)
            return node.each_with_index { |held, slot| walk(held, it_names_parameter: it_names_parameter) unless slot == 2 }
          end
          node.each { |child| walk(child, it_names_parameter: it_names_parameter) }
        end

        private

        def bare_name?(name)
          name.is_a?(Array) && name.first == :symbol_literal && token?(name[1])
        end

        def token?(node)
          node.is_a?(Array) && node.first.to_s.start_with?("@") && node[2].is_a?(Array)
        end

        def method_name(name)
          dispatch(name, :method_name) if token?(name)
        end

        def symbol(name)
          dispatch(name, :symbol)
        end

        def message(name, operator)
          return unless token?(name)
          operator_text = operator.is_a?(Array) ? operator[1] : operator&.to_s
          if operator_text.nil? && OPERATORS.include?(name[1].to_sym)
            # Operators are not colored as a method call.
          elsif (operator_text.nil? || operator_text == "::") && /\A\p{Upper}/.match?(name[1])
            # Constant-like methods are not colored as a method call.
          else
            dispatch(name, :message_name)
          end
        end

        def dispatch(name, type)
          line, column = name[2]
          @tokens << Color.send(:span, line, column, 1, type, name[1])
        end
      end

      private

      def token_type(event, text)
        case event
        when :on_kw
          VALUE_KEYWORDS.fetch(text) { :"KEYWORD_#{text.delete("?").upcase}" }
        when :on_backtick
          text == "`" ? :BACKTICK : :PERCENT_LOWER_X
        when :on_rational
          text.include?(".") ? :FLOAT_RATIONAL : :INTEGER_RATIONAL
        when :on_imaginary
          :INTEGER_IMAGINARY
        else
          EVENT_TYPES.fetch(event, event)
        end
      end

      # String content Prism reads as one piece and Ripper as several: the
      # indentation of a `<<~` heredoc line with the content after it, and
      # content Ripper breaks where a `#` does not start an interpolation.
      def joined_string_content(lexed)
        joined = []
        lexed.each do |token|
          position, event, text, state = token
          previous = joined.last
          if event == :on_tstring_content && previous && %i[on_ignored_sp on_tstring_content].include?(previous[1])
            joined[-1] = [previous[0], :on_tstring_content, previous[2] + text, state]
          elsif event == :on_ignored_sp
            joined << [position, event, text, state]
          else
            joined << token
          end
        end
        joined.map { |position, event, text, state| [position, event == :on_ignored_sp ? :on_tstring_content : event, text, state] }
      end

      # The spans a symbol colors: a `:name` symbol, the parts of a quoted or
      # interpolated one, the words of `%i[]` and `%I[]`, and a quoted label.
      def symbol_spans(lexed)
        found = []
        index = 0
        while index < lexed.size
          (line, column), event, text = lexed[index]
          case event
          when :on_symbeg
            found << span(line, column, 1, :symbol, text)
            if text == ":"
              (value_line, value_column), _, value = lexed[index + 1]
              found << span(value_line, value_column, 1, :symbol, value)
              index += 2
              next
            end
            quoted_symbol_spans(lexed, index + 1, found)
            index += 1
            next
          when :on_qsymbols_beg, :on_symbols_beg
            found << span(line, column, 1, :symbol, text)
            quoted_symbol_spans(lexed, index + 1, found)
            index += 1
            next
          when :on_tstring_beg
            closing = closing_index(lexed, index + 1)
            if closing && lexed[closing][1] == :on_label_end
              if lexed[(index + 1)...closing].any? { |_, held,| held == :on_embexpr_beg || held == :on_embvar }
                found << span(line, column, 1, :symbol, text)
                quoted_symbol_spans(lexed, index + 1, found)
                index += 1
              else
                (end_line, end_column), _, closer = lexed[closing]
                last = span(end_line, end_column, 1, :LABEL, closer)
                label = code_between(lexed, index, closing)
                found << [line, column, 1, last[3], last[4], :LABEL, label]
                index = closing + 1
              end
              next
            end
          end
          index += 1
        end
        found
      end

      # Marks the content, interpolation edges and closing of a symbol whose
      # opening is just before `index`, and answers the index after it.
      def quoted_symbol_spans(lexed, index, found)
        depth = 0
        while index < lexed.size
          (line, column), event, text = lexed[index]
          case event
          when :on_embexpr_beg
            found << span(line, column, 1, :symbol, text) if depth.zero?
            depth += 1
          when :on_embexpr_end
            depth -= 1
            found << span(line, column, 1, :symbol, text) if depth.zero?
          when :on_embvar
            found << span(line, column, 1, :symbol, text) if depth.zero?
          when :on_tstring_content
            found << span(line, column, 1, :symbol, text) if depth.zero?
          when :on_tstring_end, :on_label_end
            if depth.zero?
              found << span(line, column, 1, :symbol, text)
              return index + 1
            end
          end
          index += 1
        end
        index
      end

      # The index of the token closing the string opened just before `index`.
      def closing_index(lexed, index)
        depth = 0
        while index < lexed.size
          event = lexed[index][1]
          case event
          when :on_embexpr_beg
            depth += 1
          when :on_embexpr_end
            depth -= 1
          when :on_tstring_beg
            nested = closing_index(lexed, index + 1)
            return nil if nested.nil?
            index = nested
          when :on_tstring_end, :on_label_end
            return index if depth.zero?
          end
          index += 1
        end
        nil
      end

      def code_between(lexed, first, last)
        lexed[first..last].map { |_, _, text| text }.join
      end

      def escape_for_print(str)
        str.chars.map! { |gr|
          case gr
          when "\n"
            gr
          when "\t"
            "  "
          else
            ESCAPED_PAIRS[gr.ord] || gr
          end
        }.join
      end

      def without_circular_ref(obj, seen:, &block)
        return false if seen.key?(obj)
        seen[obj] = true
        block.call
      ensure
        seen.delete(obj)
      end
    end
  end
end
