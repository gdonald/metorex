# Ripper's scanner: Ruby source to tokens, tracking the lexer state the
# grammar and the lexer share, and handing every piece of the source,
# whitespace and comments included, to the ripper as a scanner event.

class Ripper
  module Engine
    Token = Struct.new(:type, :value, :text, :line, :column, :state, :space_before)

    # An open string: what closes it, and what may appear inside it.
    StringTerm = Struct.new(:kind, :term, :paren, :nest, :interpolate, :regexp,
                            :words, :label, :heredoc)

    # An open heredoc: its terminator and where lexing resumes on the line
    # that began it.
    Heredoc = Struct.new(:id, :dash, :squiggly, :interpolate, :resume_pos, :resume_line,
                         :resume_line_start, :at_line_start, :body_line_start)

    Embexpr = Struct.new(:saved_state, :brace_depth)

    class Scanner
      BEG = EXPR_BEG
      END_ = EXPR_END
      ENDARG = EXPR_ENDARG
      ENDFN = EXPR_ENDFN
      ARG = EXPR_ARG
      CMDARG = EXPR_CMDARG
      MID = EXPR_MID
      FNAME = EXPR_FNAME
      DOT = EXPR_DOT
      CLASS = EXPR_CLASS
      LABEL = EXPR_LABEL
      LABELED = EXPR_LABELED
      FITEM = EXPR_FITEM
      BEG_ANY = EXPR_BEG_ANY
      ARG_ANY = EXPR_ARG_ANY
      END_ANY = EXPR_END_ANY

      # Each keyword's lexer state after it, and the token type a keyword
      # with a modifier form takes when it does not begin an expression.
      KEYWORDS = {
        "__ENCODING__" => [END_, nil], "__LINE__" => [END_, nil], "__FILE__" => [END_, nil],
        "BEGIN" => [END_, nil], "END" => [END_, nil], "alias" => [FNAME | FITEM, nil],
        "and" => [BEG, nil], "begin" => [BEG, nil], "break" => [MID, nil], "case" => [BEG, nil],
        "class" => [CLASS, nil], "def" => [FNAME, nil], "defined?" => [ARG, nil], "do" => [BEG, nil],
        "else" => [BEG, nil], "elsif" => [BEG, nil], "end" => [END_, nil], "ensure" => [BEG, nil],
        "false" => [END_, nil], "for" => [BEG, nil], "if" => [BEG, :k_if_mod], "in" => [BEG, nil],
        "module" => [BEG, nil], "next" => [MID, nil], "nil" => [END_, nil], "not" => [ARG, nil],
        "or" => [BEG, nil], "redo" => [END_, nil], "rescue" => [MID, :k_rescue_mod],
        "retry" => [END_, nil], "return" => [MID, nil], "self" => [END_, nil],
        "super" => [ARG, nil], "then" => [BEG, nil], "true" => [END_, nil],
        "undef" => [FNAME | FITEM, nil], "unless" => [BEG, :k_unless_mod],
        "until" => [BEG, :k_until_mod], "when" => [BEG, nil], "while" => [BEG, :k_while_mod],
        "yield" => [ARG, nil]
      }.freeze

      PERCENT_PARENS = { "(" => ")", "[" => "]", "{" => "}", "<" => ">" }.freeze

      # Global variables named by one punctuation character.
      SPECIAL_GVARS = "~*$?!@/\\;,.=:<>\"_0".bytes.freeze

      attr_accessor :state, :command_start, :lpar_beg, :in_argdef, :in_kwarg
      attr_reader :paren_nest, :line, :end_seen

      def initialize(emitter, source, line)
        @emitter = emitter
        @src = source.b
        @encoding = source.encoding
        @len = @src.bytesize
        @pos = 0
        @line = line
        @line_start = 0
        @state = BEG
        @command_start = true
        @modes = []
        @cond_stack = 0
        @cmdarg_stack = 0
        @paren_nest = 0
        @lpar_beg = -1
        @in_argdef = false
        @in_kwarg = false
        @heredoc_end = nil
        @pending_newline = nil
        @end_seen = false
        @token_seen = false
        @heredoc_indent = nil
        skip_shebang
      end

      def cond_push(bit) = @cond_stack = (@cond_stack << 1) | (bit ? 1 : 0)
      def cond_pop = @cond_stack >>= 1
      def cond_p = @cond_stack.anybits?(1)
      def cmdarg_push(bit) = @cmdarg_stack = (@cmdarg_stack << 1) | (bit ? 1 : 0)
      def cmdarg_pop = @cmdarg_stack >>= 1
      def cmdarg_p = @cmdarg_stack.anybits?(1)

      # The width a squiggly heredoc's lines were dedented by, which the
      # grammar reads once the heredoc's string is complete.
      def take_heredoc_indent
        indent = @heredoc_indent
        @heredoc_indent = nil
        indent
      end

      def next_token
        mode = @modes.last
        if mode.is_a?(StringTerm)
          string_token(mode)
        elsif mode.is_a?(Heredoc)
          heredoc_token(mode)
        elsif mode == :dvar
          @modes.pop
          code_token
        else
          code_token
        end
      end

      # Every remaining scanner event, after a syntax error ends the parse.
      def drain
        loop do
          token = next_token
          break if token.type == :eof
        end
      rescue StandardError
        nil
      end

      private

      def skip_shebang
        return unless @src.start_with?("\xEF\xBB\xBF".b) && @encoding == Encoding::UTF_8

        @pos = 3
        @line_start = 3
      end

      # A magic comment naming the source's encoding sets the encoding of
      # every token after it.
      def encoding=(name)
        @encoding = Encoding.find(name)
      rescue ArgumentError
        nil
      end

      def peek(offset = 0) = @src.getbyte(@pos + offset)

      def text(from, to = @pos) = @src.byteslice(from, to - from).force_encoding(@encoding)

      def column_of(position) = position - @line_start

      def emit(event, from, to = @pos, line: @line, line_start: @line_start, state: @state)
        @emitter.scan_event(event, text(from, to), line, from - line_start, state)
      end

      def token(type, event, from, value_text: nil, space: false)
        value = emit(event, from)
        Token.new(type, value, value_text || text(from), @line, from - @line_start, @state, space)
      end

      def ident_start?(byte)
        byte && ((byte >= 97 && byte <= 122) || (byte >= 65 && byte <= 90) || byte == 95 || byte >= 128)
      end

      def ident_char?(byte)
        byte && (ident_start?(byte) || digit?(byte))
      end

      def digit?(byte) = byte && byte >= 48 && byte <= 57

      def space?(byte) = [32, 9, 12, 13, 11].include?(byte)

      def upper?(byte) = byte && byte >= 65 && byte <= 90

      def beg? = @state.anybits?(BEG_ANY) || @state.allbits?(ARG | LABELED)
      def end? = @state.anybits?(END_ANY)
      def arg? = @state.anybits?(ARG_ANY)
      def after_operator? = @state.anybits?(FNAME | DOT)
      def label_possible?(cmd_state) = (@state.anybits?(LABEL | ENDFN) && !cmd_state) || arg?

      def spcarg?(space_seen, byte) = arg? && space_seen && !space?(byte) && byte != 10

      def label_suffix?(offset = 0) = peek(offset) == 58 && peek(offset + 1) != 58

      def at_line_start?(position = @pos) = position == @line_start

      # Moves past the newline at the current position, onto the line
      # after any heredoc bodies that line began.
      def cross_newline
        @pos += 1
        if @heredoc_end
          @pos, @line = @heredoc_end
          @heredoc_end = nil
        else
          @line += 1
        end
        @line_start = @pos
      end

      def code_token
        space_seen = false
        last_state = @state
        cmd_state = @command_start
        @command_start = false
        loop do
          last_state = @state
          start = @pos
          byte = peek
          case byte
          when nil, 0, 4, 26
            flush_newline(:on_nl) if @pending_newline
            return Token.new(:eof, nil, "", @line, @pos - @line_start, @state, space_seen)
          when 13
            if peek(1) == 10
              return newline_token(start, 2) if significant_newline?

              ignore_newline(start, 2)
              return keyword_argument_newline if keyword_argument_newline?

              next
            end
            @pos += 1 while space?(peek) && !(peek == 13 && peek(1) == 10)
            emit(:on_sp, start)
            space_seen = true
            next
          when 32, 9, 12, 11
            @pos += 1 while space?(peek) && !(peek == 13 && peek(1) == 10)
            emit(:on_sp, start)
            space_seen = true
            next
          when 35
            @pos += 1 until peek.nil? || peek == 10
            had_newline = !peek.nil?
            @pos += 1 if had_newline
            unless @token_seen
              coding = @emitter.magic_comment(text(start))
              self.encoding = coding if coding
            end
            emit(:on_comment, start, line: @line)
            if had_newline
              @pos -= 1
              result = after_comment_newline(cmd_state)
              return result if result

              space_seen = false
            end
            next
          when 10
            return newline_token(start, 1) if significant_newline?

            ignore_newline(start, 1)
            return keyword_argument_newline if keyword_argument_newline?

            next
          when 92
            width = if peek(1) == 10
                      2
                    elsif peek(1) == 13 && peek(2) == 10
                      3
                    end
            if width
              line = @line
              line_start = @line_start
              @pos += width - 1
              cross_newline
              @emitter.scan_event(:on_sp, text(start, start + width), line, start - line_start, @state)
              space_seen = true
              next
            end
            @pos += 1
            return error_token(start, "backslash appearing outside of a string")
          end
          if at_line_start? && byte == 61 && @src.byteslice(@pos, 6) == "=begin" &&
             (space?(peek(6)) || peek(6) == 10 || peek(6).nil?)
            embedded_document
            next
          end
          if at_line_start? && byte == 95 && @src.byteslice(@pos, 7) == "__END__" &&
             [10, 13, nil].include?(peek(7))
            @pos += 7
            @pos += 1 if peek == 13
            @pos += 1 if peek == 10
            emit(:on___end__, start)
            @end_seen = true
            @pos = @len
            return Token.new(:eof, nil, "", @line, 0, @state, space_seen)
          end
          @token_seen = true
          return operator_or_word(byte, start, space_seen, cmd_state, last_state)
        end
      end

      def significant_newline?
        ignored = @state.anybits?(BEG | CLASS | FNAME | DOT) && !@state.anybits?(LABELED)
        !ignored && !@state.allbits?(ARG | LABELED)
      end

      # After a label with no value, where keyword arguments or a hash
      # pattern may end, the newline is scanned as ignored but still ends
      # the statement.
      def keyword_argument_newline?
        @in_kwarg && @state.allbits?(ARG | LABELED) &&
          !(@state.anybits?(BEG | CLASS | FNAME | DOT) && !@state.anybits?(LABELED))
      end

      def keyword_argument_newline
        @state = BEG
        @command_start = true
        Token.new(:nl, nil, "\n", @line - 1, 0, @state, false)
      end

      def ignore_newline(start, width)
        line = @line
        line_start = @line_start
        @pos += width - 1
        cross_newline
        @emitter.scan_event(:on_ignored_nl, text(start, start + width), line, start - line_start, @state)
      end

      # A newline that may end the statement: it does, unless the next
      # code line begins with a leading `.` or `&.`.
      def newline_token(start, width)
        @pending_newline = [start, start + width, @line, @line_start]
        @pos += width - 1
        cross_newline
        resolve_newline
      end

      def after_comment_newline(_cmd_state)
        unless significant_newline?
          cross_newline
          return keyword_argument_newline if keyword_argument_newline?

          return nil
        end
        @pending_newline ||= [nil, nil, @line, @line_start]
        cross_newline
        resolve_newline
      end

      # Looks past blank space and comment lines for a leading dot. Comments
      # found on the way are scanned as they are reached.
      def resolve_newline
        loop do
          start = @pos
          @pos += 1 while space?(peek) && !(peek == 13 && peek(1) == 10)
          byte = peek
          if byte == 35
            emit(:on_sp, start) if @pos > start
            comment_start = @pos
            @pos += 1 until peek.nil? || peek == 10
            had_newline = !peek.nil?
            @pos += 1 if had_newline
            emit(:on_comment, comment_start)
            @pos -= 1 if had_newline
            break unless had_newline

            cross_newline
            next
          end
          leading_dot = (byte == 46 && peek(1) != 46) || (byte == 38 && peek(1) == 46)
          if leading_dot
            flush_newline(:on_ignored_nl)
            emit(:on_sp, start) if @pos > start
            return code_token_after_dot
          end
          @pos = start
          break
        end
        @state = BEG
        @command_start = true
        newline = @pending_newline
        flush_newline(:on_nl)
        Token.new(:nl, nil, "\n", newline[2], newline[0] ? newline[0] - newline[3] : 0, @state, false)
      end

      def code_token_after_dot
        code_token
      end

      def flush_newline(event)
        start, finish, line, line_start = @pending_newline
        @pending_newline = nil
        return unless start

        @emitter.scan_event(event, text(start, finish), line, start - line_start, @state)
      end

      def embedded_document
        start = @pos
        @pos += 1 until peek.nil? || peek == 10
        @pos += 1 unless peek.nil?
        emit(:on_embdoc_beg, start)
        @line += 1 if @src.getbyte(@pos - 1) == 10
        @line_start = @pos
        loop do
          return compile_error("embedded document meets end of file") if peek.nil?

          start = @pos
          @pos += 1 until peek.nil? || peek == 10
          @pos += 1 unless peek.nil?
          finished = @src.byteslice(start, 4) == "=end" &&
                     (space?(@src.getbyte(start + 4)) || [10, 13, nil].include?(@src.getbyte(start + 4)))
          emit(finished ? :on_embdoc_end : :on_embdoc, start)
          @line += 1 if @src.getbyte(@pos - 1) == 10
          @line_start = @pos
          return if finished
        end
      end

      def compile_error(message)
        @emitter.compile_error(message)
        nil
      end

      def error_token(start, message)
        Token.new(:error, message, text(start), @line, start - @line_start, @state, false)
      end

      def set_state_after_operator = @state = after_operator? ? ARG : BEG

      def op(type, start, space_seen, event = :on_op)
        token(type, event, start, space: space_seen)
      end

      def op_assign(start, space_seen)
        @state = BEG
        token(:op_asgn, :on_op, start, space: space_seen)
      end

      def operator_or_word(byte, start, space_seen, cmd_state, last_state)
        case byte
        when 42 # *
          if peek(1) == 42
            if peek(2) == 61
              @pos += 3
              return op_assign(start, space_seen)
            end
            @pos += 2
            type = spcarg?(space_seen, peek) || beg? ? :dstar : :pow
            set_state_after_operator
            return op(type, start, space_seen)
          end
          if peek(1) == 61
            @pos += 2
            return op_assign(start, space_seen)
          end
          @pos += 1
          type = spcarg?(space_seen, peek) || beg? ? :star : :*
          set_state_after_operator
          op(type, start, space_seen)
        when 33 # !
          @pos += 1
          if after_operator?
            @state = ARG
            if peek == 64
              @pos += 1
              return op(:"!", start, space_seen)
            end
          else
            @state = BEG
          end
          if peek == 61
            @pos += 1
            return op(:"!=", start, space_seen)
          end
          if peek == 126
            @pos += 1
            return op(:"!~", start, space_seen)
          end
          op(:"!", start, space_seen)
        when 61 # =
          @pos += 1
          set_state_after_operator
          if peek == 61
            @pos += 1
            if peek == 61
              @pos += 1
              return op(:"===", start, space_seen)
            end
            return op(:"==", start, space_seen)
          end
          if peek == 126
            @pos += 1
            return op(:"=~", start, space_seen)
          end
          if peek == 62
            @pos += 1
            return op(:"=>", start, space_seen)
          end
          op(:"=", start, space_seen)
        when 60 # <
          if peek(1) == 60 && !@state.anybits?(DOT | CLASS) && !end? &&
             (!arg? || @state.anybits?(LABELED) || space_seen)
            heredoc = heredoc_start(start, space_seen)
            return heredoc if heredoc
          end
          @pos += 1
          if after_operator?
            @state = ARG
          else
            @command_start = true if @state.anybits?(CLASS)
            @state = BEG
          end
          if peek == 61
            @pos += 1
            if peek == 62
              @pos += 1
              return op(:"<=>", start, space_seen)
            end
            return op(:"<=", start, space_seen)
          end
          if peek == 60
            @pos += 1
            if peek == 61
              @pos += 1
              return op_assign(start, space_seen)
            end
            return op(:"<<", start, space_seen)
          end
          op(:<, start, space_seen)
        when 62 # >
          @pos += 1
          set_state_after_operator
          if peek == 61
            @pos += 1
            return op(:">=", start, space_seen)
          end
          if peek == 62
            @pos += 1
            if peek == 61
              @pos += 1
              return op_assign(start, space_seen)
            end
            return op(:">>", start, space_seen)
          end
          op(:>, start, space_seen)
        when 34 # "
          @pos += 1
          label = label_possible?(cmd_state)
          @modes << StringTerm.new(:string, 34, nil, 0, true, false, nil, label)
          token(:string_beg, :on_tstring_beg, start, space: space_seen)
        when 96 # `
          @pos += 1
          if @state.anybits?(FNAME)
            @state = ENDFN
            return op(:"`", start, space_seen, :on_backtick)
          end
          if @state.anybits?(DOT)
            @state = cmd_state ? CMDARG : ARG
            return op(:"`", start, space_seen, :on_backtick)
          end
          @modes << StringTerm.new(:xstring, 96, nil, 0, true, false, nil, false)
          token(:xstring_beg, :on_backtick, start, space: space_seen)
        when 39 # '
          @pos += 1
          label = label_possible?(cmd_state)
          @modes << StringTerm.new(:string, 39, nil, 0, false, false, nil, label)
          token(:string_beg, :on_tstring_beg, start, space: space_seen)
        when 63 # ?
          question_mark(start, space_seen)
        when 38 # &
          if peek(1) == 38
            @state = BEG
            if peek(2) == 61
              @pos += 3
              return op_assign(start, space_seen)
            end
            @pos += 2
            return op(:"&&", start, space_seen)
          end
          if peek(1) == 61
            @pos += 2
            return op_assign(start, space_seen)
          end
          if peek(1) == 46
            @pos += 2
            @state = DOT
            return op(:"&.", start, space_seen)
          end
          @pos += 1
          type = spcarg?(space_seen, peek) || beg? ? :amper : :&
          set_state_after_operator
          op(type, start, space_seen)
        when 124 # |
          if peek(1) == 124
            @state = BEG
            if peek(2) == 61
              @pos += 3
              return op_assign(start, space_seen)
            end
            if last_state.anybits?(BEG)
              @pos += 1
              return op(:|, start, space_seen)
            end
            @pos += 2
            return op(:"||", start, space_seen)
          end
          if peek(1) == 61
            @pos += 2
            return op_assign(start, space_seen)
          end
          @pos += 1
          @state = after_operator? ? ARG : BEG | LABEL
          op(:|, start, space_seen)
        when 43 # +
          sign(start, space_seen, :+)
        when 45 # -
          sign(start, space_seen, :-)
        when 46 # .
          dot(start, space_seen, last_state)
        when 48..57
          number(start, space_seen)
        when 41 # )
          @pos += 1
          cond_pop
          cmdarg_pop
          @state = ENDFN
          @paren_nest -= 1
          token(:")", :on_rparen, start, space: space_seen)
        when 93 # ]
          @pos += 1
          cond_pop
          cmdarg_pop
          @state = END_
          @paren_nest -= 1
          token(:"]", :on_rbracket, start, space: space_seen)
        when 125 # }
          close_brace(start, space_seen)
        when 58 # :
          colon(start, space_seen)
        when 47 # /
          if beg?
            @pos += 1
            @modes << StringTerm.new(:regexp, 47, nil, 0, true, true, nil, false)
            return token(:regexp_beg, :on_regexp_beg, start, space: space_seen)
          end
          if peek(1) == 61
            @pos += 2
            return op_assign(start, space_seen)
          end
          if spcarg?(space_seen, peek(1))
            @pos += 1
            @modes << StringTerm.new(:regexp, 47, nil, 0, true, true, nil, false)
            return token(:regexp_beg, :on_regexp_beg, start, space: space_seen)
          end
          @pos += 1
          set_state_after_operator
          op(:/, start, space_seen)
        when 94 # ^
          if peek(1) == 61
            @pos += 2
            return op_assign(start, space_seen)
          end
          @pos += 1
          set_state_after_operator
          op(:^, start, space_seen)
        when 59 # ;
          @pos += 1
          @state = BEG
          @command_start = true
          token(:";", :on_semicolon, start, space: space_seen)
        when 44 # ,
          @pos += 1
          @state = BEG | LABEL
          token(:",", :on_comma, start, space: space_seen)
        when 126 # ~
          @pos += 1
          if after_operator?
            @pos += 1 if peek == 64
            @state = ARG
          else
            @state = BEG
          end
          op(:~, start, space_seen)
        when 40 # (
          @pos += 1
          type = if beg?
                   :lparen
                 elsif !space_seen
                   :"("
                 elsif arg? || @state.allbits?(END_ | LABEL)
                   :lparen_arg
                 else
                   :"("
                 end
          @paren_nest += 1
          cond_push(false)
          cmdarg_push(false)
          @state = BEG | LABEL
          token(type, :on_lparen, start, space: space_seen)
        when 91 # [
          @pos += 1
          @paren_nest += 1
          if after_operator?
            if peek == 93
              @pos += 1
              if peek == 61
                @pos += 1
                @state = ARG
                return op(:aset, start, space_seen)
              end
              @state = ARG
              return op(:aref, start, space_seen)
            end
            @state = ARG | LABEL
            return token(:"[", :on_lbracket, start, space: space_seen)
          end
          type = if beg? || (arg? && (space_seen || @state.anybits?(LABELED)))
                   :lbrack
                 else
                   :"["
                 end
          @state = BEG | LABEL
          cond_push(false)
          cmdarg_push(false)
          token(type, :on_lbracket, start, space: space_seen)
        when 123 # {
          open_brace(start, space_seen)
        when 37 # %
          percent(start, space_seen, cmd_state)
        when 36 # $
          global_variable(start, space_seen, last_state)
        when 64 # @
          instance_variable(start, space_seen, last_state)
        else
          if ident_start?(byte)
            identifier(start, space_seen, cmd_state, last_state)
          else
            @pos += 1
            error_token(start, "Invalid char '#{text(start)}' in expression")
          end
        end
      end

      def sign(start, space_seen, operator)
        @pos += 1
        name = operator.to_s
        if after_operator?
          @state = ARG
          if peek == 64
            @pos += 1
            return op(:"#{name}@", start, space_seen)
          end
          return op(operator, start, space_seen)
        end
        if peek == 61
          @pos += 1
          return op_assign(start, space_seen)
        end
        if operator == :- && peek == 62
          @pos += 1
          @state = ENDFN
          return token(:lambda, :on_tlambda, start, space: space_seen)
        end
        if beg? || spcarg?(space_seen, peek)
          @state = BEG
          if digit?(peek)
            return number(start, space_seen) if operator == :+

            return op(:uminus_num, start, space_seen)
          end
          return op(operator == :+ ? :uplus : :uminus, start, space_seen)
        end
        @state = BEG
        op(operator, start, space_seen)
      end

      def dot(start, space_seen, last_state)
        was_beg = beg?
        @state = BEG
        if peek(1) == 46
          if peek(2) == 46
            @pos += 3
            if @in_argdef
              @state = ENDARG
              return op(:bdot3, start, space_seen)
            end
            if @lpar_beg >= 0 && @lpar_beg + 1 == @paren_nest && last_state.anybits?(LABEL)
              return op(:dot3, start, space_seen)
            end
            return op(was_beg ? :bdot3 : :dot3, start, space_seen)
          end
          @pos += 2
          return op(was_beg ? :bdot2 : :dot2, start, space_seen)
        end
        @pos += 1
        if digit?(peek)
          return error_token(start, "no .<digit> floating literal anymore; put 0 before dot")
        end

        @state = DOT
        token(:".", :on_period, start, space: space_seen)
      end

      def open_brace(start, space_seen)
        @pos += 1
        embexpr = @modes.last
        embexpr.brace_depth += 1 if embexpr.is_a?(Embexpr)
        if @lpar_beg == @paren_nest
          @state = BEG
          @command_start = true
          @lpar_beg = -1
          @paren_nest += 1
          cond_push(false)
          cmdarg_push(false)
          return token(:lambeg, :on_tlambeg, start, space: space_seen)
        end
        type = if @state.anybits?(LABELED)
                 :lbrace
               elsif @state.anybits?(ARG_ANY | END_ | ENDFN)
                 :"{"
               elsif @state.anybits?(ENDARG)
                 :lbrace_arg
               else
                 :lbrace
               end
        if type == :lbrace
          @state = BEG | LABEL
        else
          @command_start = true
          @state = BEG
        end
        @paren_nest += 1
        cond_push(false)
        cmdarg_push(false)
        token(type, :on_lbrace, start, space: space_seen)
      end

      def close_brace(start, space_seen)
        @pos += 1
        embexpr = @modes.last
        if embexpr.is_a?(Embexpr)
          if embexpr.brace_depth.zero?
            @modes.pop
            value = emit(:on_embexpr_end, start)
            result = Token.new(:string_dend, value, "}", @line, start - @line_start, @state, space_seen)
            @state = embexpr.saved_state
            return result
          end
          embexpr.brace_depth -= 1
        end
        cond_pop
        cmdarg_pop
        @state = END_
        @paren_nest -= 1
        token(:"}", :on_rbrace, start, space: space_seen)
      end

      def colon(start, space_seen)
        if peek(1) == 58
          @pos += 2
          if beg? || @state.anybits?(CLASS) || (arg? && space_seen)
            @state = BEG
            return op(:colon3, start, space_seen)
          end
          @state = DOT
          return op(:colon2, start, space_seen)
        end
        following = peek(1)
        if end? || following.nil? || space?(following) || following == 10 || following == 35
          @pos += 1
          @state = BEG
          return op(:":", start, space_seen)
        end
        if following == 34 || following == 39
          @pos += 2
          @modes << StringTerm.new(:dsym, following, nil, 0, following == 34, false, nil, false)
          @state = FNAME
          return token(:symbeg, :on_symbeg, start, space: space_seen)
        end
        @pos += 1
        @state = FNAME
        token(:symbeg, :on_symbeg, start, space: space_seen)
      end

      def question_mark(start, space_seen)
        if end?
          @pos += 1
          @state = BEG
          return op(:"?", start, space_seen)
        end
        following = peek(1)
        if following.nil?
          @pos += 1
          return error_token(start, "incomplete character syntax")
        end
        if space?(following) || following == 10
          @pos += 1
          @state = BEG
          return op(:"?", start, space_seen)
        end
        if (ident_char?(following) && following < 128) && ident_char?(peek(2))
          @pos += 1
          @state = BEG
          return op(:"?", start, space_seen)
        end
        @pos += 1
        if peek == 92
          skip_escape
        else
          @pos += char_width(peek)
        end
        @state = END_
        token(:char, :on_CHAR, start, space: space_seen)
      end

      def char_width(byte)
        return 1 if byte.nil? || byte < 0x80
        return 2 if byte < 0xE0
        return 3 if byte < 0xF0

        4
      end

      # Moves past one backslash escape, as written.
      def skip_escape
        @pos += 1
        byte = peek
        case byte
        when nil
          nil
        when 117 # u
          @pos += 1
          if peek == 123
            @pos += 1 until peek.nil? || peek == 125
            @pos += 1 unless peek.nil?
          else
            4.times { @pos += 1 if peek && hex?(peek) }
          end
        when 120 # x
          @pos += 1
          2.times { @pos += 1 if peek && hex?(peek) }
        when 48..55
          3.times { @pos += 1 if peek && peek >= 48 && peek <= 55 }
        when 67, 99, 77 # C c M
          if byte == 99
            @pos += 1
          else
            @pos += 1
            @pos += 1 if peek == 45
          end
          if peek == 92
            skip_escape
          else
            @pos += char_width(peek)
          end
        else
          @pos += char_width(byte)
        end
      end

      def hex?(byte) = digit?(byte) || (byte >= 97 && byte <= 102) || (byte >= 65 && byte <= 70)

      def number(start, space_seen)
        @state = END_
        @pos += 1 if peek == 43 || peek == 45
        type = :int
        if peek == 48 && [120, 88, 98, 66, 111, 79, 100, 68, 95].include?(peek(1))
          @pos += 2
          @pos += 1 while peek && (hex?(peek) || peek == 95)
        elsif peek == 48 && digit?(peek(1))
          @pos += 1 while digit?(peek) || peek == 95
        else
          @pos += 1 while digit?(peek) || peek == 95
          if peek == 46 && digit?(peek(1))
            type = :float
            @pos += 1
            @pos += 1 while digit?(peek) || peek == 95
          end
          if (peek == 101 || peek == 69) && (digit?(peek(1)) || ((peek(1) == 43 || peek(1) == 45) && digit?(peek(2))))
            type = :float
            @pos += 2
            @pos += 1 while digit?(peek) || peek == 95
            return finish_number(start, space_seen, type, exponent: true)
          end
        end
        finish_number(start, space_seen, type)
      end

      def finish_number(start, space_seen, type, exponent: false)
        if peek == 114 && !exponent && !ident_char?(peek(1)) || (peek == 114 && peek(1) == 105 && !exponent && !ident_char?(peek(2)))
          @pos += 1
          type = :rational
        end
        if peek == 105 && !ident_char?(peek(1))
          @pos += 1
          type = :imaginary
        end
        token(type, :"on_#{type == :int ? 'int' : type}", start, space: space_seen)
      end

      def global_variable(start, space_seen, last_state)
        @pos += 1
        byte = peek
        type = :gvar
        event = :on_gvar
        if byte == 95 && ident_char?(peek(1))
          @pos += 1 while ident_char?(peek)
        elsif [38, 96, 39, 43].include?(byte)
          @pos += 1
          unless last_state.anybits?(FNAME)
            type = :backref
            event = :on_backref
          end
        elsif byte == 45
          @pos += 1
          @pos += 1 if ident_char?(peek)
        elsif byte == 48
          @pos += 1
        elsif digit?(byte)
          @pos += 1 while digit?(peek)
          unless last_state.anybits?(FNAME)
            type = :backref
            event = :on_backref
          end
        elsif SPECIAL_GVARS.include?(byte)
          @pos += 1
        elsif ident_start?(byte)
          @pos += 1 while ident_char?(peek)
        else
          @state = END_
          return error_token(start, "'$' without identifiers is not allowed as a global variable name")
        end
        @state = END_
        token(type, event, start, space: space_seen)
      end

      def instance_variable(start, space_seen, last_state)
        @pos += 1
        type = :ivar
        event = :on_ivar
        if peek == 64
          @pos += 1
          type = :cvar
          event = :on_cvar
        end
        unless ident_start?(peek)
          @state = END_
          return error_token(start, "'#{text(start)}' without identifiers is not allowed as #{type == :ivar ? 'an instance' : 'a class'} variable name")
        end
        @pos += 1 while ident_char?(peek)
        @state = last_state.anybits?(FNAME) ? ENDFN : END_
        token(type, event, start, space: space_seen)
      end

      def identifier(start, space_seen, cmd_state, last_state)
        @pos += 1 while ident_char?(peek)
        type = nil
        byte = peek
        if byte == 33 || byte == 63
          if peek(1) != 61
            @pos += 1
            type = :fid
          elsif @state.anybits?(FNAME) && peek(2) != 126 && peek(2) != 62 && (peek(2) != 61 || peek(3) == 62)
            @pos += 1
            type = :ident
          end
        elsif byte == 61 && @state.anybits?(FNAME) && peek(1) != 126 && peek(1) != 62 &&
              (peek(1) != 61 || peek(2) == 62)
          @pos += 1
          type = :ident
        end
        name = text(start)
        type ||= constant_name?(start) ? :const : :ident
        if label_possible?(cmd_state) && label_suffix?
          @pos += 1
          @state = ARG | LABELED
          return token(:label, :on_label, start, space: space_seen)
        end
        if !@state.anybits?(DOT) && (keyword = KEYWORDS[name])
          return keyword_token(name, keyword, start, space_seen)
        end

        @state = if @state.anybits?(BEG_ANY | ARG_ANY | DOT)
                   cmd_state ? CMDARG : ARG
                 elsif @state == FNAME
                   ENDFN
                 else
                   END_
                 end
        if !last_state.anybits?(DOT | FNAME) && type == :ident &&
           (@emitter.local?(name) || name.match?(/\A_[1-9]\z/))
          @state = END_ | LABEL
        end
        event = type == :const ? :on_const : :on_ident
        token(type, event, start, space: space_seen)
      end

      def constant_name?(start)
        first = @src.getbyte(start)
        return upper?(first) if first < 128

        character = @src.byteslice(start, char_width(first)).force_encoding(@encoding)
        character.valid_encoding? && character.match?(/\A\p{Upper}/)
      end

      def keyword_token(name, keyword, start, space_seen)
        state_before = @state
        if state_before.anybits?(FNAME)
          @state = ENDFN
          return token(:"k_#{name}", :on_kw, start, space: space_seen)
        end
        @state = keyword[0]
        @command_start = true if @state.anybits?(BEG)
        if name == "do"
          if @lpar_beg == @paren_nest
            @lpar_beg = -1
            return token(:k_do_lambda, :on_kw, start, space: space_seen)
          end
          return token(:k_do_cond, :on_kw, start, space: space_seen) if cond_p
          if cmdarg_p && !state_before.anybits?(CMDARG)
            return token(:k_do_block, :on_kw, start, space: space_seen)
          end

          return token(:k_do, :on_kw, start, space: space_seen)
        end
        return token(:"k_#{name}", :on_kw, start, space: space_seen) if state_before.anybits?(BEG | LABELED | CLASS)

        if keyword[1]
          @state = BEG | LABEL
          return token(keyword[1], :on_kw, start, space: space_seen)
        end
        token(:"k_#{name}", :on_kw, start, space: space_seen)
      end

      def percent(start, space_seen, cmd_state)
        if beg?
          return quotation(start, space_seen)
        end
        if peek(1) == 61
          @pos += 2
          return op_assign(start, space_seen)
        end
        if spcarg?(space_seen, peek(1)) || (@state.anybits?(FITEM) && peek(1) == 115)
          return quotation(start, space_seen)
        end

        @pos += 1
        set_state_after_operator
        op(:%, start, space_seen)
      end

      def quotation(start, space_seen)
        @pos += 1
        kind = peek
        alphanumeric = kind && ((kind >= 97 && kind <= 122) || (kind >= 65 && kind <= 90) || digit?(kind))
        if !alphanumeric
          kind_letter = "Q"
        else
          kind_letter = kind.chr
          @pos += 1
        end
        opener = peek
        opener_alphanumeric = opener && ((opener >= 97 && opener <= 122) || (opener >= 65 && opener <= 90) || digit?(opener))
        if opener.nil? || opener_alphanumeric || opener >= 128
          return error_token(start, "unknown type of %string")
        end
        @pos += 1
        closer = PERCENT_PARENS[opener.chr]&.ord || opener
        paren = closer == opener ? nil : opener
        case kind_letter
        when "Q"
          @modes << StringTerm.new(:string, closer, paren, 0, true, false, nil, false)
          token(:string_beg, :on_tstring_beg, start, space: space_seen)
        when "q"
          @modes << StringTerm.new(:string, closer, paren, 0, false, false, nil, false)
          token(:string_beg, :on_tstring_beg, start, space: space_seen)
        when "W"
          @modes << StringTerm.new(:words, closer, paren, 0, true, false, :words, false)
          token(:words_beg, :on_words_beg, start, space: space_seen)
        when "w"
          @modes << StringTerm.new(:qwords, closer, paren, 0, false, false, :qwords, false)
          token(:qwords_beg, :on_qwords_beg, start, space: space_seen)
        when "I"
          @modes << StringTerm.new(:symbols, closer, paren, 0, true, false, :symbols, false)
          token(:symbols_beg, :on_symbols_beg, start, space: space_seen)
        when "i"
          @modes << StringTerm.new(:qsymbols, closer, paren, 0, false, false, :qsymbols, false)
          token(:qsymbols_beg, :on_qsymbols_beg, start, space: space_seen)
        when "x"
          @modes << StringTerm.new(:xstring, closer, paren, 0, true, false, nil, false)
          token(:xstring_beg, :on_backtick, start, space: space_seen)
        when "r"
          @modes << StringTerm.new(:regexp, closer, paren, 0, true, true, nil, false)
          token(:regexp_beg, :on_regexp_beg, start, space: space_seen)
        when "s"
          @modes << StringTerm.new(:dsym, closer, paren, 0, false, false, nil, false)
          @state = FNAME | FITEM
          token(:symbeg, :on_symbeg, start, space: space_seen)
        else
          error_token(start, "unknown type of %string")
        end
      end

      # The next piece of an open string: its content up to an
      # interpolation or its end, an interpolation's opening, or its close.
      def string_token(mode)
        start = @pos
        start_line = @line
        start_line_start = @line_start
        if mode.words
          if space?(peek) || peek == 10
            if peek == 10
              cross_newline
              return words_separator(start, start_line, start_line_start)
            end
            @pos += 1 while space?(peek)
            return words_separator(start, start_line, start_line_start)
          end
        end
        byte = peek
        return unterminated(mode) if byte.nil?

        if byte == mode.term && mode.nest.zero?
          return close_string(mode, start)
        end
        if mode.interpolate && byte == 35
          interpolation = interpolation_token(start)
          return interpolation if interpolation
        end
        string_content(mode)
        if @pos == start
          return close_string(mode, start) if peek == mode.term

          return unterminated(mode)
        end
        value = @emitter.scan_event(:on_tstring_content, text(start), start_line, start - start_line_start, @state)
        Token.new(:string_content, value, text(start), start_line, start - start_line_start, @state, false)
      end

      def words_separator(start, line, line_start)
        value = @emitter.scan_event(:on_words_sep, text(start), line, start - line_start, @state)
        Token.new(:words_sep, value, text(start), line, start - line_start, @state, false)
      end

      def unterminated(mode)
        @modes.pop
        message = mode.regexp ? "unterminated regexp meets end of file" : "unterminated string meets end of file"
        error_token(@pos, message)
      end

      def interpolation_token(start)
        following = peek(1)
        if following == 123
          @pos += 2
          value = emit(:on_embexpr_beg, start)
          result = Token.new(:string_dbeg, value, '#{', @line, start - @line_start, @state, false)
          @modes << Embexpr.new(@state, 0)
          @state = BEG
          @command_start = true
          return result
        end
        variable = (following == 64 && (ident_start?(peek(2)) || (peek(2) == 64 && ident_start?(peek(3))))) ||
                   (following == 36 && (ident_start?(peek(2)) || SPECIAL_GVARS.include?(peek(2)) || digit?(peek(2)) ||
                                        [38, 96, 39, 43, 45].include?(peek(2))))
        return nil unless variable

        @pos += 1
        value = emit(:on_embvar, start)
        result = Token.new(:string_dvar, value, "#", @line, start - @line_start, @state, false)
        @modes << :dvar
        @state = BEG
        result
      end

      # A string's content up to its end or an interpolation. Content
      # stops before every `#{`, `#$` and `#@`, and one that does not
      # begin an interpolation starts the next piece of content.
      def string_content(mode)
        content_start = @pos
        loop do
          byte = peek
          break if byte.nil?

          if mode.words && (space?(byte) || byte == 10)
            break
          end
          if mode.paren && byte == mode.paren
            mode.nest += 1
          elsif byte == mode.term
            break if mode.nest.zero?

            mode.nest -= 1
          elsif mode.interpolate && byte == 35 && @pos > content_start && [123, 36, 64].include?(peek(1))
            break
          elsif byte == 92
            if peek(1) == 10
              @pos += 1
              cross_newline
              next
            end
            if mode.interpolate && !mode.words
              skip_escape
            else
              @pos += 1
              @pos += char_width(peek) unless peek.nil?
            end
            next
          elsif byte == 10
            cross_newline
            next
          end
          @pos += 1
        end
      end

      def interpolation_follows?
        following = peek(1)
        return true if following == 123
        return ident_start?(peek(2)) || (peek(2) == 64 && ident_start?(peek(3))) if following == 64
        return ident_start?(peek(2)) || SPECIAL_GVARS.include?(peek(2)) || digit?(peek(2)) || [38, 96, 39, 43, 45].include?(peek(2)) if following == 36

        false
      end

      def close_string(mode, start)
        @pos += 1
        @modes.pop
        if mode.regexp
          @pos += 1 while peek && "imxounse".include?(peek.chr)
          value = emit(:on_regexp_end, start)
          result = Token.new(:regexp_end, value, text(start), @line, start - @line_start, @state, false)
          @state = END_
          return result
        end
        if mode.label && label_suffix?
          @pos += 1
          @state = ARG | LABELED
          return token(:label_end, :on_label_end, start)
        end
        @state = END_
        token(:string_end, :on_tstring_end, start)
      end

      # `<<ID`, `<<-ID` or `<<~ID`, with the identifier bare or quoted.
      def heredoc_start(start, space_seen)
        position = @pos + 2
        dash = squiggly = false
        if @src.getbyte(position) == 45
          dash = true
          position += 1
        elsif @src.getbyte(position) == 126
          squiggly = true
          position += 1
        end
        quote = @src.getbyte(position)
        interpolate = true
        xstring = false
        if [39, 34, 96].include?(quote)
          close = @src.index(quote.chr, position + 1)
          return nil unless close && !@src.byteslice(position + 1, close - position - 1).include?("\n")

          id = @src.byteslice(position + 1, close - position - 1)
          interpolate = quote != 39
          xstring = quote == 96
          position = close + 1
        else
          return nil unless ident_char?(quote)

          id_start = position
          position += 1 while ident_char?(@src.getbyte(position))
          id = @src.byteslice(id_start, position - id_start)
        end
        @pos = position
        value = emit(:on_heredoc_beg, start)
        result = Token.new(xstring ? :xstring_beg : :string_beg, value, text(start), @line,
                           start - @line_start, @state, space_seen)
        line_end = @src.index("\n", @pos)
        body_start = @heredoc_end ? @heredoc_end[0] : (line_end ? line_end + 1 : @len)
        body_line = @heredoc_end ? @heredoc_end[1] : @line + 1
        heredoc = Heredoc.new(id, dash || squiggly, squiggly, interpolate, @pos, @line, @line_start,
                              true, nil)
        heredoc.resume_pos = @pos
        @modes << heredoc
        @saved_heredoc_end = @heredoc_end
        @pos = body_start
        @line = body_line
        @line_start = body_start
        @heredoc_lines = [] if squiggly
        result
      end

      def heredoc_terminator_at?(heredoc, position)
        scan = position
        if heredoc.dash
          scan += 1 while space?(@src.getbyte(scan))
        end
        return false unless @src.byteslice(scan, heredoc.id.bytesize) == heredoc.id

        after = scan + heredoc.id.bytesize
        after += 1 if @src.getbyte(after) == 13
        [10, nil].include?(@src.getbyte(after))
      end

      def heredoc_token(heredoc)
        if @pos >= @len && !heredoc_terminator_at?(heredoc, @pos)
          @modes.pop
          restore_after_heredoc(heredoc)
          return error_token(@pos, "can't find string \"#{heredoc.id}\" anywhere before EOF")
        end
        if at_line_start? && heredoc_terminator_at?(heredoc, @pos)
          start = @pos
          @pos += 1 until peek.nil? || peek == 10
          @pos += 1 unless peek.nil?
          value = emit(:on_heredoc_end, start)
          result = Token.new(:string_end, value, text(start), @line, 0, @state, false)
          @state = END_
          @line += 1 if @src.getbyte(@pos - 1) == 10
          @modes.pop
          @heredoc_indent = heredoc_indent_width if heredoc.squiggly
          restore_after_heredoc(heredoc)
          return result
        end
        start = @pos
        start_line = @line
        start_line_start = @line_start
        if heredoc.interpolate && peek == 35
          interpolation = interpolation_token(start)
          return interpolation if interpolation
        end
        loop do
          byte = peek
          break if byte.nil?

          if heredoc.interpolate && byte == 35 && @pos > start && [123, 36, 64].include?(peek(1))
            break
          elsif byte == 92 && heredoc.interpolate
            @pos += 1
            if peek == 10
              cross_heredoc_line(heredoc)
              break if heredoc.squiggly || heredoc_terminator_at?(heredoc, @pos)

              next
            end
            @pos += char_width(peek) unless peek.nil?
            next
          elsif byte == 10
            cross_heredoc_line(heredoc)
            break if heredoc.squiggly || heredoc_terminator_at?(heredoc, @pos) || @pos >= @len

            next
          end
          @pos += 1
        end
        content = text(start)
        @heredoc_lines << [content, start - start_line_start] if heredoc.squiggly
        value = @emitter.scan_event(:on_tstring_content, content, start_line, start - start_line_start, @state)
        Token.new(:string_content, value, content, start_line, start - start_line_start, @state, false)
      end

      def cross_heredoc_line(_heredoc)
        @pos += 1
        @line += 1
        @line_start = @pos
      end

      # The smallest indentation among a squiggly heredoc's lines, counting
      # a tab to the next multiple of eight and skipping blank lines.
      def heredoc_indent_width
        widths = []
        @heredoc_lines.each do |content, column|
          next unless column.zero?

          width = 0
          blank = false
          content.each_byte do |byte|
            if byte == 32
              width += 1
            elsif byte == 9
              width = (width / 8 + 1) * 8
            else
              blank = byte == 10 || byte == 13
              break
            end
          end
          widths << width unless blank
        end
        widths.min || 0
      end

      def restore_after_heredoc(heredoc)
        @heredoc_end = [@pos, @line]
        @pos = heredoc.resume_pos
        @line = heredoc.resume_line
        @line_start = heredoc.resume_line_start
      end
    end
  end
end
