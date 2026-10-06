# Ripper's grammar: a recursive descent parser over the scanner's tokens.
# It builds each statement as a tree of parser events, then dispatches the
# events bottom up, the order a bottom-up parser reduces them in.

class Ripper
  module Engine
    # A parser event not yet dispatched. A command or a multiple
    # assignment is `terminal`: no binary operator can follow it. `last` is
    # the index of the last token the grammar had taken when it built the
    # event.
    Ev = Struct.new(:name, :args, :terminal, :last)

    # A value already dispatched, such as a squiggly heredoc's contents,
    # which are dedented as soon as the heredoc ends. The parts it was built
    # from and the width taken off each line stay beside it.
    Dispatched = Struct.new(:value, :source, :width)

    # A syntax error, raised at the token that cannot continue the parse.
    class SyntaxFailure < StandardError
      attr_accessor :token
    end

    # One method's or block's local variables, the anonymous parameters a
    # method declared, and how a block refers to its arguments.
    Scope = Struct.new(:names, :block, :parent, :anonymous, :ordinary_parameters, :numbered, :it_used)

    class Grammar
      BINARY = {
        "||": 4, "&&": 5,
        "<=>": 7, "==": 7, "===": 7, "!=": 7, "=~": 7, "!~": 7,
        ">": 8, ">=": 8, "<": 8, "<=": 8,
        "|": 9, "^": 9, "&": 10, "<<": 11, ">>": 11,
        "+": 12, "-": 12, "*": 13, "/": 13, "%": 13, pow: 15
      }.freeze

      NONASSOCIATIVE = %i[<=> == === != =~ !~].freeze

      LITERAL_KEYWORDS = %i[k_nil k_true k_false k_self k___FILE__ k___LINE__ k___ENCODING__].freeze

      ARGUMENT_START = (%i[
        int float rational imaginary char ident const fid ivar gvar cvar backref label
        string_beg xstring_beg regexp_beg words_beg qwords_beg symbols_beg qsymbols_beg symbeg
        lbrack lparen_arg lparen star dstar amper uminus_num uminus uplus ! ~ colon3 lambda
        bdot2 bdot3 k_def k_if k_unless k_while k_until k_case k_for k_begin k_class k_module
        k_defined? k_yield k_super k_not k_redo k_retry k_BEGIN k_END
      ] + LITERAL_KEYWORDS).freeze

      STATEMENT_ENDS = [
        :eof, :k_end, :"}", :")", :"]", :k_else, :k_elsif, :k_when, :k_in, :k_rescue, :k_ensure,
        :k_then, :string_dend
      ].freeze

      OPERATOR_NAMES = %i[
        | ^ & <=> == === =~ !~ != > >= < <= << >> + - * / % pow ! ~ +@ -@ aref aset ` !@ ~@ star dstar
        uplus uminus uminus_num amper
      ].freeze

      def initialize(bridge, scanner)
        @bridge = bridge
        @scanner = scanner
        @lookahead = nil
        @scope = Scope.new({}, false, nil, [])
        @in_def = false
        @in_class = false
        @loop_depth = 0
        @in_rescue = false
        @paren_depth = 0
        @command_argument_depth = 0
        @block_depth = 0
        @in_defined = false
        @jump_errors = []
        @reported_errors = []
        @taken = []
      end

      # The tree of events for the whole program, not dispatched, with the
      # tokens it was built from in the order they were taken.
      def syntax_tree
        [program, @taken]
      end

      def run
        tree = program
        emit(tree)
      rescue SyntaxFailure => failure
        @bridge.parse_error(failure.message, failure.token || @lookahead)
        @scanner.drain
        nil
      end

      def local?(name)
        scope = @scope
        while scope
          return true if scope.names.key?(name)
          break unless scope.block

          scope = scope.parent
        end
        false
      end

      private

      # Tokens

      def peek
        @lookahead ||= begin
          token = @scanner.next_token
          raise SyntaxFailure, token.value if token.type == :error

          token
        end
      end

      def peek_type = peek.type

      def take
        token = peek
        @lookahead = nil
        token.index = @taken.size
        @taken << token
        token
      end

      def accept(type)
        peek_type == type ? take : nil
      end

      def expect(type, *alternatives)
        return take if peek_type == type || alternatives.include?(peek_type)

        unexpected
      end

      def unexpected
        token = peek
        found = token.type == :eof ? "end-of-input" : "'#{token.text}'"
        failure = SyntaxFailure.new("syntax error, unexpected #{found}")
        failure.token = token
        raise failure
      end

      def skip_newlines
        take while peek_type == :nl
      end

      def term? = peek_type == :";" || peek_type == :nl

      def skip_terms
        take while term?
      end

      def ev(name, *args) = Ev.new(name, args, false, @taken.size - 1)

      def terminal(name, *args) = Ev.new(name, args, true, @taken.size - 1)

      # Dispatch

      def emit(node)
        case node
        when Ev
          @bridge.parser_event(node.name, *node.args.map { |arg| emit(arg) })
        when Token
          node.value
        when Array
          node.map { |item| emit(item) }
        when Dispatched
          node.value
        else
          node
        end
      end

      # Scopes

      def declare(name)
        @scope.names[name] = true
      end

      def method_scope
        scope = @scope
        scope = scope.parent while scope.block && scope.parent
        scope
      end

      # The compile errors reported, each as [message, first, last, at]:
      # the indexes of the first and last token the error names, or nil when
      # MRI names none, and the index of the last token taken when it was
      # reported.
      attr_reader :reported_errors
      public :reported_errors

      def report_error(message, first = nil, last = first, at: last || @taken.size - 1)
        @bridge.compile_error(message)
        @reported_errors << [message, first, last, at]
      end

      # The indexes of the first and last token a node was written with.
      def token_bounds(node)
        case node
        when Token then [node.index, node.index]
        when Ev
          bounds = node.args.map { |arg| token_bounds(arg) }.compact
          bounds.empty? ? nil : [bounds.map(&:first).min, bounds.map(&:last).max]
        end
      end

      def anonymous_argument(kind, message)
        report_error(message) unless method_scope.anonymous.include?(kind)
      end

      def numbered_parameter
        if @scope.ordinary_parameters
          report_error("ordinary parameter is defined")
        elsif @scope.it_used
          report_error("numbered parameters are not allowed when 'it' is already used")
        end
        @scope.numbered = true
      end

      def it_parameter
        if @scope.ordinary_parameters
          report_error("ordinary parameter is defined")
        elsif @scope.numbered
          report_error("'it' is not allowed when a numbered parameter is already used")
        end
        @scope.it_used = true
      end

      # A method, class or module body starts with fresh `do` and
      # condition stacks in the scanner, as a block body starts a fresh
      # `do` stack.
      def with_scope(block)
        saved = @scope
        @scope = Scope.new({}, block, saved, [])
        @scanner.cmdarg_push(false)
        @scanner.cond_push(false) unless block
        yield
      ensure
        @scanner.cmdarg_pop
        @scanner.cond_pop unless block
        @scope = saved
      end

      def in_method_body
        saved = [@in_def, @loop_depth, @in_rescue, @jump_errors]
        @in_def = true
        @loop_depth = 0
        @in_rescue = false
        @jump_errors = []
        yield
      ensure
        @in_def, @loop_depth, @in_rescue, @jump_errors = saved
      end

      # A singleton class opened inside a method may `return` from that
      # method. A class or module body may not.
      def in_class_body(singleton: false)
        saved = [@in_def, @in_class, @loop_depth, @in_rescue, @block_depth, @jump_errors, @method_around_class]
        @method_around_class = singleton && (@in_def || @method_around_class)
        @in_def = false
        @in_class = true
        @loop_depth = 0
        @in_rescue = false
        @block_depth = 0
        @jump_errors = []
        yield
      ensure
        @in_def, @in_class, @loop_depth, @in_rescue, @block_depth, @jump_errors, @method_around_class = saved
      end

      def in_loop
        @loop_depth += 1
        yield
      ensure
        @loop_depth -= 1
      end

      def in_block(&)
        @block_depth += 1
        in_loop(&)
      ensure
        @block_depth -= 1
      end

      # Statements

      def program
        stmts = statements
        unexpected unless peek_type == :eof
        ev(:program, stmts)
      end

      def statements
        saved = [@command_argument_depth, @command_in_parentheses]
        @command_argument_depth = 0
        @command_in_parentheses = false
        statement_list
      ensure
        @command_argument_depth, @command_in_parentheses = saved
      end

      def statement_list
        list = ev(:stmts_new)
        count = 0
        if term?
          list = ev(:stmts_add, list, ev(:void_stmt))
          count += 1
          skip_terms
        end
        until STATEMENT_ENDS.include?(peek_type)
          list = ev(:stmts_add, list, statement)
          count += 1
          break unless term?

          skip_terms
        end
        list = ev(:stmts_add, list, ev(:void_stmt)) if count.zero?
        list
      end

      def statement
        @jump_errors.push([])
        node = case peek_type
               when :k_alias then alias_statement
               when :k_undef then undef_statement
               when :k_BEGIN, :k_END then begin_end_block
               else expression_statement
               end
        modifiers(node)
      ensure
        errors = @jump_errors.pop
        if @jump_errors.empty?
          errors.each { |message, first, last| report_error(message, first, last, at: @taken.size - 1) }
        else
          @jump_errors.last.concat(errors)
        end
      end

      # A jump outside a loop, reported unless the statement holding it
      # turns out to be the body of a `while` or `until` modifier.
      # The error is reported once the outermost statement holding the jump
      # has ended. The entry it answers names the jump's tokens.
      def invalid_jump(message, keyword)
        entry = [message, keyword.index, keyword.index]
        if @jump_errors.empty?
          report_error(*entry)
        else
          @jump_errors.last << entry
        end
        entry
      end

      def modifiers(node)
        loop do
          case peek_type
          when :k_if_mod
            take
            node = ev(:if_mod, expr_value, node)
          when :k_unless_mod
            take
            node = ev(:unless_mod, expr_value, node)
          when :k_while_mod
            take
            @jump_errors.last&.clear
            node = ev(:while_mod, expr_value, node)
          when :k_until_mod
            take
            @jump_errors.last&.clear
            node = ev(:until_mod, expr_value, node)
          when :k_rescue_mod
            take
            node = ev(:rescue_mod, node, expression_statement)
          else
            return node
          end
        end
      end

      def fitem
        token = peek
        case token.type
        when :symbeg
          symbol
        else
          name = method_name(fname: true)
          ev(:symbol_literal, name)
        end
      end

      # A method's name after a dot, or, as `fname`, where a method is
      # defined or aliased: an operator there leaves the scanner as after a
      # method name.
      def method_name(fname: false)
        token = peek
        if %i[ident const fid].include?(token.type) || token.type.to_s.start_with?("k_") || token.type == :label
          return take
        end
        if OPERATOR_NAMES.include?(token.type)
          take
          @scanner.state = EXPR_ENDFN if fname
          return token
        end

        unexpected
      end

      def alias_statement
        take
        if %i[gvar backref].include?(peek_type)
          first = take
          second = expect(:gvar, :backref)
          if second.type == :backref && second.text.match?(/\A\$\d/)
            report_error("can't make alias for the number variables", second.index)
          end
          return ev(:var_alias, first, second)
        end
        first = fitem
        @scanner.state = EXPR_FNAME | EXPR_FITEM
        second = fitem
        ev(:alias, first, second)
      end

      def undef_statement
        take
        items = [fitem]
        while peek_type == :","
          take
          @scanner.state = EXPR_FNAME | EXPR_FITEM
          items << fitem
        end
        ev(:undef, items)
      end

      def begin_end_block
        keyword = take
        expect(:lbrace, :"{", :lbrace_arg)
        stmts = statements
        expect(:"}")
        ev(keyword.text.to_sym, stmts)
      end

      def expression_statement
        if peek_type == :star
          return multiple_assignment(nil)
        end

        expr(command: true, mlhs: true)
      end

      def expr_value = expr(command: true)

      def expr(command: false, mlhs: false)
        left = not_expression(command: command, mlhs: mlhs)
        while %i[k_and k_or].include?(peek_type)
          operator = take.text.to_sym
          left = ev(:binary, left, operator, not_expression(command: true))
        end
        left
      end

      def not_expression(command:, mlhs: false)
        if peek_type == :k_not
          take
          return ev(:unary, :not, not_expression(command: true)) unless peek_type == :"("

          node = postfix(not_call, command: command)
          return node if node.terminal

          return binary(node, 0)
        end
        value = arg(0, command: command, mlhs: mlhs)
        return value if value.is_a?(Ev) && value.terminal

        if command && %i[=> k_in].include?(peek_type)
          take
          pattern = in_pattern_context { pattern_top_body }
          return terminal(:case, value, ev(:in, pattern, nil, nil))
        end
        value
      end

      # Arguments and operators

      def arg(min_precedence = 0, command: false, mlhs: false)
        left = unary(command: command, mlhs: mlhs)
        return left if left.is_a?(Ev) && left.terminal

        binary(left, min_precedence)
      end

      def binary(left, min_precedence)
        loop do
          type = peek_type
          if type == :"?"
            break if min_precedence > 2

            take
            skip_newlines
            consequent = arg(0)
            skip_newlines
            if peek_type == :label
              raise SyntaxFailure, "syntax error, unexpected label"
            end
            expect(:":")
            skip_newlines
            alternative = arg(2)
            left = ev(:ifop, left, consequent, alternative)
          elsif type == :dot2 || type == :dot3
            break if min_precedence > 3

            take
            right = argument_start?(true) ? arg(4) : nil
            left = ev(type, left, right)
          elsif (precedence = BINARY[type])
            break if precedence < min_precedence

            operator = take
            right_precedence = type == :pow ? precedence : precedence + 1
            right = arg(right_precedence)
            name = type == :pow ? :** : type
            declare_named_captures(left) if type == :"=~"
            left = ev(:binary, left, name, right)
            if NONASSOCIATIVE.include?(type) && NONASSOCIATIVE.include?(peek_type)
              unexpected
            end
            operator
          else
            break
          end
        end
        left
      end

      # A literal regexp on the left of `=~` declares a local for each of
      # its named groups.
      def declare_named_captures(node)
        return unless node.is_a?(Ev) && node.name == :regexp_literal

        parts = []
        list = node.args[0]
        while list.is_a?(Ev) && list.name == :regexp_add
          parts.unshift(list.args[1])
          list = list.args[0]
        end
        return unless parts.all? { |part| part.is_a?(Token) }

        parts.map(&:text).join.scan(/\(\?<([A-Za-z_]\w*)>/) { |(name)| declare(name) }
      end

      def argument_start?(range_end = false)
        type = peek_type
        return false if range_end && %i[k_then k_do k_do_cond].include?(type)

        ARGUMENT_START.include?(type) || %i[lbrace].include?(type)
      end

      def unary(command: false, mlhs: false)
        case peek_type
        when :uminus
          take
          ev(:unary, :-@, arg(14))
        when :uplus
          take
          ev(:unary, :+@, arg(16))
        when :"!"
          take
          ev(:unary, :!, arg(16, command: command))
        when :~
          take
          ev(:unary, :~, arg(16))
        when :uminus_num
          take
          number = take
          if peek_type == :pow
            take
            ev(:unary, :-@, ev(:binary, number, :**, arg(15)))
          else
            postfix(ev(:unary, :-@, number), command: command, mlhs: mlhs)
          end
        when :k_defined?
          take
          skip_newlines
          if peek_type == :"(" || peek_type == :lparen_arg || peek_type == :lparen
            take
            skip_newlines
            saved = @in_defined
            @in_defined = true
            value = begin
              expr(command: true)
            ensure
              @in_defined = saved
            end
            skip_newlines
            expect(:")")
            return postfix(ev(:defined, value), command: command, mlhs: mlhs)
          end
          ev(:defined, arg(7))
        when :bdot2, :bdot3
          type = take.type
          ev(type == :bdot2 ? :dot2 : :dot3, nil, arg(4))
        else
          operand(command: command, mlhs: mlhs)
        end
      end

      # A primary expression with its calls, and the assignment it may be
      # the target of.
      def operand(command:, mlhs:)
        node = postfix(primary(command: command), command: command, mlhs: mlhs)
        return node if node.is_a?(Ev) && node.terminal

        case peek_type
        when :"="
          return node unless assignable?(node)

          take
          target = assignment_target(node)
          if command && peek_type == :star
            return terminal(:assign, target, splat_rhs)
          end

          value = assignment_value(command)
          if command && peek_type == :"," && !(value.is_a?(Ev) && value.terminal)
            take
            value = multiple_rhs(ev(:args_add, ev(:args_new), value))
            return terminal(:assign, target, value)
          end
          Ev.new(:assign, [target, value], value.is_a?(Ev) && value.terminal, @taken.size - 1)
        when :op_asgn
          return node unless assignable?(node)

          operator = take
          target = assignment_target(node, declare_local: true)
          value = assignment_value(command)
          Ev.new(:opassign, [target, operator, value], value.is_a?(Ev) && value.terminal, @taken.size - 1)
        else
          node
        end
      end

      def assignment_value(command)
        value = arg(0, command: command)
        if value.is_a?(Ev) && value.terminal
          return value unless peek_type == :k_rescue_mod

          take
          return terminal(:rescue_mod, value, expr(command: true))
        end

        if peek_type == :k_rescue_mod
          take
          value = ev(:rescue_mod, value, arg(0))
        end
        value
      end

      def splat_rhs
        take
        list = ev(:mrhs_add_star, ev(:mrhs_new), arg(0))
        while accept(:",")
          if peek_type == :star
            take
            list = ev(:mrhs_add_star, list, arg(0))
          else
            list = ev(:mrhs_add, list, arg(0))
          end
        end
        list
      end

      def multiple_rhs(args)
        loop do
          star = peek_type == :star
          take if star
          value = arg(0)
          unless peek_type == :","
            return ev(star ? :mrhs_add_star : :mrhs_add, ev(:mrhs_new_from_args, args), value)
          end

          take
          args = ev(star ? :args_add_star : :args_add, args, value)
        end
      end

      def assignable?(node)
        return true if node.is_a?(Token) && %i[ivar gvar cvar const].include?(node.type)
        return false unless node.is_a?(Ev)

        case node.name
        when :vcall, :var_ref
          token = node.args[0]
          %i[ident ivar gvar cvar const].include?(token.type)
        when :call
          node.args[2].is_a?(Token)
        when :aref, :const_path_ref, :top_const_ref
          true
        else
          false
        end
      end

      def assignment_target(node, declare_local: true)
        case node.name
        when :vcall, :var_ref
          token = node.args[0]
          declare(token.text) if declare_local && token.type == :ident
          if token.type == :const && @in_def
            report_error("dynamic constant assignment", token.index)
          end
          ev(:var_field, token)
        when :call
          receiver, operator, name = node.args
          if operator.is_a?(Token) && operator.type == :colon2 && name.type == :const
            ev(:const_path_field, receiver, name)
          else
            ev(:field, receiver, operator, name)
          end
        when :aref
          ev(:aref_field, *node.args)
        when :const_path_ref
          report_error("dynamic constant assignment", *token_bounds(node)) if @in_def
          ev(:const_path_field, *node.args)
        when :top_const_ref
          report_error("dynamic constant assignment", node.args[0].index - 1, node.args[0].index) if @in_def
          ev(:top_const_field, *node.args)
        end
      end

      # Multiple assignment

      def multiple_assignment(first)
        list = ev(:mlhs_new)
        list = ev(:mlhs_add, list, first) if first
        list = mlhs_items(list, first.nil?)
        return terminal(:mlhs_in_paren, list) if peek_type == :")" && @paren_depth.positive?

        expect(:"=")
        terminal(:massign, list, massign_rhs)
      end

      def mlhs_items(list, at_item)
        splatted = false
        loop do
          if at_item
            break if peek_type == :"=" || peek_type == :")" || peek_type == :k_in || peek_type == :"|"

            if peek_type == :star
              take
              target = [:",", :"=", :")", :k_in, :|].include?(peek_type) ? nil : mlhs_target
              list = ev(:mlhs_add_star, list, target)
              splatted = true
            elsif splatted
              post = ev(:mlhs_add, ev(:mlhs_new), mlhs_item)
              while peek_type == :"," && take
                post = ev(:mlhs_add, post, mlhs_item)
              end
              list = ev(:mlhs_add_post, list, post)
              break
            else
              list = ev(:mlhs_add, list, mlhs_item)
            end
          end
          break unless accept(:",")

          at_item = true
        end
        list
      end

      def mlhs_item
        if peek_type == :lparen || peek_type == :"(" || peek_type == :lparen_arg
          node = parenthesized(nested: true)
          return node if node.name == :mlhs_paren

          node = postfix(node)
          unexpected unless assignable?(node)
          return assignment_target(node)
        end
        mlhs_target
      end

      def mlhs_target
        node = postfix(primary(command: false), command: false)
        unexpected unless assignable?(node)
        assignment_target(node)
      end

      def massign_rhs
        if peek_type == :star
          take
          first = ev(:args_add_star, ev(:args_new), arg(0))
          return ev(:mrhs_add_star, ev(:mrhs_new), first.args[1]) unless peek_type == :","

          take
          return multiple_rhs(first)
        end
        value = arg(0, command: true)
        return value if value.is_a?(Ev) && value.terminal

        if peek_type == :","
          take
          return multiple_rhs(ev(:args_add, ev(:args_new), value))
        end
        if peek_type == :k_rescue_mod
          take
          value = ev(:rescue_mod, value, arg(0))
        end
        value
      end

      # Primaries

      def primary(command:)
        token = peek
        case token.type
        when :int, :float, :rational, :imaginary, :char, :backref
          take
        when :string_beg
          strings
        when :xstring_beg
          xstring
        when :regexp_beg
          regexp
        when :words_beg, :qwords_beg, :symbols_beg, :qsymbols_beg
          words
        when :symbeg
          symbol
        when :ident, :fid
          identifier(command)
        when :const
          constant(command)
        when :ivar, :gvar, :cvar
          ev(:var_ref, take)
        when *LITERAL_KEYWORDS
          ev(:var_ref, take)
        when :colon3
          take
          name = expect(:const, :ident)
          if peek_type == :"("
            return ev(:method_add_arg, ev(:fcall, name), paren_args)
          end

          ev(:top_const_ref, name)
        when :lbrack
          array_literal
        when :lbrace
          hash_literal
        when :lparen
          parenthesized
        when :lparen_arg
          take
          skip_newlines
          if peek_type == :")"
            @scanner.state = EXPR_ENDARG
            take
            return ev(:paren, ev(:stmts_add, ev(:stmts_new), ev(:void_stmt)))
          end
          stmts = statements
          @scanner.state = EXPR_ENDARG
          expect(:")")
          ev(:paren, stmts)
        when :lambda
          lambda_literal
        when :k_if then if_expression
        when :k_unless then unless_expression
        when :k_while then loop_expression(:while)
        when :k_until then loop_expression(:until)
        when :k_case then case_expression
        when :k_for then for_expression
        when :k_begin then begin_expression
        when :k_def then def_expression
        when :k_class then class_expression
        when :k_module then module_expression
        when :k_return then return_expression(command)
        when :k_break, :k_next then jump_expression(command)
        when :k_redo
          keyword = take
          invalid_jump("Invalid redo", keyword) if @loop_depth.zero? && !@in_defined
          ev(:redo)
        when :k_retry
          keyword = take
          report_error("Invalid retry without rescue", keyword.index) unless @in_rescue || @in_defined
          ev(:retry)
        when :k_yield then yield_expression(command)
        when :k_super then super_expression(command)
        when :k_not
          take
          not_call
        when :k_defined?
          unary
        when :k___END__
          unexpected
        else
          unexpected
        end
      end

      def not_call
        expect(:"(", :lparen, :lparen_arg)
        skip_newlines
        if peek_type == :")"
          take
          return ev(:unary, :not, nil)
        end
        value = expr(command: true)
        skip_newlines
        expect(:")")
        ev(:unary, :not, value)
      end

      def identifier(command)
        name = take
        if peek_type == :"("
          return ev(:method_add_arg, ev(:fcall, name), paren_args)
        end
        if name.type == :ident && !command_follows?(command)
          return ev(:var_ref, name) if local?(name.text) && !block_follows?

          if @scope.block && name.text.match?(/\A_[1-9]\z/)
            numbered_parameter
            return ev(:var_ref, name)
          end
          it_parameter if @scope.block && name.text == "it" && !block_follows?
        end
        if command_follows?(command)
          args = command_args
          return command_block(terminal(:command, name, args))
        end
        if block_follows?
          return ev(:method_add_arg, ev(:fcall, name), ev(:args_new))
        end
        return ev(:method_add_arg, ev(:fcall, name), ev(:args_new)) if name.type == :fid

        ev(:vcall, name)
      end

      def constant(command)
        name = take
        if peek_type == :"("
          return ev(:method_add_arg, ev(:fcall, name), paren_args)
        end
        if command_follows?(command)
          return command_block(terminal(:command, name, command_args))
        end
        if block_follows?
          return ev(:method_add_arg, ev(:fcall, name), ev(:args_new))
        end

        ev(:var_ref, name)
      end

      def command_follows?(command)
        command && peek.space_before && argument_start?
      end

      def block_follows?
        peek_type == :"{" || peek_type == :k_do
      end

      def parenthesized(nested: false)
        take
        skip_newlines
        if peek_type == :")"
          take
          return ev(:paren, ev(:stmts_add, ev(:stmts_new), ev(:void_stmt)))
        end
        @paren_depth += 1
        begin
          stmts = paren_statements
        ensure
          @paren_depth -= 1
        end
        if stmts.is_a?(Ev) && stmts.name == :mlhs_paren
          return nested ? stmts : mlhs_paren_result(stmts)
        end

        skip_newlines
        expect(:")")
        ev(:paren, stmts)
      end

      # A parenthesized multiple assignment target: it must lead into an
      # assignment.
      def mlhs_paren_result(paren)
        if peek_type == :"="
          take
          return terminal(:massign, paren, massign_rhs)
        end
        list = ev(:mlhs_add, ev(:mlhs_new), paren)
        list = mlhs_items(list, false) if peek_type == :","
        return terminal(:mlhs_in_paren, list) if peek_type == :")" && @paren_depth.positive?

        expect(:"=")
        terminal(:massign, list, massign_rhs)
      end

      def paren_statements
        saved = @command_in_parentheses
        @command_in_parentheses = false
        paren_statement_list
      ensure
        @command_in_parentheses = saved
      end

      def paren_statement_list
        list = ev(:stmts_new)
        count = 0
        if term?
          list = ev(:stmts_add, list, ev(:void_stmt))
          count += 1
          skip_terms
        end
        until STATEMENT_ENDS.include?(peek_type)
          node = case peek_type
                 when :k_alias then alias_statement
                 when :k_undef then undef_statement
                 when :star then multiple_assignment(nil)
                 else expr(command: true, mlhs: true)
                 end
          if count.zero? && node.is_a?(Ev) && node.name == :mlhs_in_paren
            inner = node.args[0]
            expect(:")")
            return ev(:mlhs_paren, inner)
          end
          list = ev(:stmts_add, list, modifiers(node))
          count += 1
          break unless term?

          skip_terms
        end
        list = ev(:stmts_add, list, ev(:void_stmt)) if count.zero?
        list
      end

      # Calls

      def postfix(node, command: false, mlhs: false)
        loop do
          case peek_type
          when :".", :"&."
            receiver_is_block_call = block_call?(node)
            operator = take
            skip_newlines
            if peek_type == :"("
              node = ev(:method_add_arg, ev(:call, node, operator, :call), paren_args)
              next
            end
            name = method_name
            if receiver_is_block_call
              node = block_call_rest(node, operator, name)
              next
            end
            node = call_rest(ev(:call, node, operator, name), node, operator, name, command)
            return node if node.terminal && !block_call?(node)
          when :colon2
            operator = take
            name = method_name
            if name.type == :const && peek_type != :"(" && !command_follows?(command)
              node = ev(:const_path_ref, node, name)
              next
            end
            node = call_rest(ev(:call, node, operator, name), node, operator, name, command)
            return node if node.terminal
          when :"["
            take
            skip_newlines
            args = peek_type == :"]" ? nil : call_args(:"]")
            skip_newlines
            expect(:"]")
            node = ev(:aref, node, args)
          when :"{"
            return node unless block_target?(node)

            node = ev(:method_add_block, block_receiver(node), brace_block)
          when :k_do
            return node unless block_target?(node)

            node = ev(:method_add_block, block_receiver(node), do_block)
          when :","
            if mlhs && assignable?(node)
              return multiple_assignment(assignment_target(node))
            end

            return node
          else
            return node
          end
        end
      end

      # A command given a `do` block, which a call may follow.
      def block_call?(node)
        node.is_a?(Ev) && node.terminal && node.name == :method_add_block && node.args[1].name == :do_block
      end

      def block_opener? = %i[{ lbrace_arg k_do k_do_block].include?(peek_type)

      # A call on a command given a `do` block. A block after it makes it a
      # command_call, and arguments without parentheses are added to a call.
      def block_call_rest(receiver, operator, name)
        arguments = peek_type == :"(" ? paren_args : nil
        if block_opener?
          block = %i[{ lbrace_arg].include?(peek_type) ? brace_block : do_block
          return terminal(:method_add_block, ev(:command_call, receiver, operator, name, arguments), block)
        end
        call = ev(:call, receiver, operator, name)
        return ev(:method_add_arg, call, arguments) if arguments
        return terminal(:method_add_arg, call, command_args) if command_follows?(true)

        call
      end

      def block_target?(node)
        return false unless node.is_a?(Ev)

        %i[method_add_arg call vcall fcall command command_call super zsuper].include?(node.name)
      end

      def block_receiver(node)
        return ev(:method_add_arg, ev(:fcall, node.args[0]), ev(:args_new)) if node.name == :vcall

        node
      end

      def call_rest(call, receiver, operator, name, command)
        if peek_type == :"("
          return ev(:method_add_arg, call, paren_args)
        end
        if command_follows?(command)
          args = command_args
          node = terminal(:command_call, receiver, operator, name, args)
          return command_block(node)
        end
        call
      end

      def paren_args
        saved_depth = @command_argument_depth
        @command_argument_depth = 0
        parenthesized_arguments
      ensure
        @command_argument_depth = saved_depth
      end

      def parenthesized_arguments
        take
        skip_newlines
        if peek_type == :")"
          take
          return ev(:arg_paren, nil)
        end
        if peek_type == :bdot3
          take
          if peek_type == :")"
            take
            return ev(:arg_paren, ev(:args_forward))
          end
          first = ev(:dot3, nil, arg(4))
          args = if peek_type == :","
                   take
                   skip_newlines
                   call_args_after(ev(:args_add, ev(:args_new), first))
                 else
                   ev(:args_add_block, ev(:args_add, ev(:args_new), first), false)
                 end
          skip_newlines
          expect(:")")
          return ev(:arg_paren, args)
        end
        args = call_args(:")", command: true, paren: true)
        skip_newlines
        expect(:")")
        ev(:arg_paren, args)
      end

      def command_args
        lookahead_pushed = [:"(", :lparen, :lparen_arg, :"[", :lbrack].include?(peek_type)
        if lookahead_pushed
          @scanner.cmdarg_pop
          @scanner.cmdarg_push(true)
          @scanner.cmdarg_push(false)
        else
          @scanner.cmdarg_push(true)
        end
        @command_argument_depth += 1
        begin
          args = call_args(nil, command: true)
        ensure
          @command_argument_depth -= 1
        end
        if peek_type == :lbrace_arg
          @scanner.cmdarg_pop
          @scanner.cmdarg_pop
          @scanner.cmdarg_push(false)
        else
          @scanner.cmdarg_pop
        end
        args
      end

      # The first argument, which may be a command. A command written as
      # the argument in parentheses takes no `do` block.
      def command_argument(paren)
        saved = @command_in_parentheses
        @command_in_parentheses = paren
        arg(0, command: true)
      ensure
        @command_in_parentheses = saved
      end

      def command_block(node)
        return node if @command_argument_depth.positive?
        return node if @command_in_parentheses && peek_type == :k_do_block

        if peek_type == :k_do_block
          return terminal(:method_add_block, node, do_block)
        end
        if peek_type == :lbrace_arg
          return terminal(:method_add_block, node, brace_block)
        end

        node
      end

      # The arguments of a call, closed by `closer` or ended by whatever
      # cannot continue them.
      def call_args_after(args) = call_args(:")", paren: true, start: args)

      def call_args(closer, command: false, paren: false, start: nil)
        args = start || ev(:args_new)
        assocs = nil
        block = false
        has_block = false
        trailing_comma = false
        loop do
          case peek_type
          when :amper
            take
            block = argument_start? ? arg(0) : nil
            anonymous_argument(:block, "no anonymous block parameter") if block.nil?
            has_block = true
            break
          when :star
            take
            value = argument_start? ? arg(0) : nil
            anonymous_argument(:rest, "no anonymous rest parameter") if value.nil?
            args = ev(:args_add_star, args, value)
          when :dstar
            take
            value = argument_start? ? arg(0) : nil
            anonymous_argument(:keyword_rest, "no anonymous keyword rest parameter") if value.nil?
            (assocs ||= []) << ev(:assoc_splat, value)
          when :label
            label = take
            value = argument_start? ? arg(0) : nil
            (assocs ||= []) << ev(:assoc_new, label, value)
          when :bdot3
            if paren
              take
              if peek_type == :")"
                return ev(:args_add, args, ev(:args_forward))
              end

              args = ev(:args_add, args, ev(:dot3, nil, arg(4)))
            else
              args = ev(:args_add, args, arg(0))
            end
          else
            value = if command && args.name == :args_new && assocs.nil?
                      command_argument(paren)
                    else
                      arg(0)
                    end
            if value.is_a?(Ev) && value.terminal
              return ev(:args_add, args, value)
            end

            if value.is_a?(Ev) && value.name == :__label_string
              (assocs ||= []) << ev(:assoc_new, value.args[0], argument_start? ? arg(0) : nil)
            elsif peek_type == :"=>"
              take
              skip_newlines
              (assocs ||= []) << ev(:assoc_new, value, arg(0))
            else
              args = ev(:args_add, args, value)
            end
          end
          break unless peek_type == :","

          take
          skip_newlines
          if closer && peek_type == closer
            trailing_comma = true
            break
          end
        end
        args = ev(:args_add, args, ev(:bare_assoc_hash, assocs)) if assocs
        return args if trailing_comma && !has_block

        ev(:args_add_block, args, block)
      end

      def aref_args
        args = ev(:args_new)
        assocs = nil
        loop do
          skip_newlines
          break if peek_type == :"]"

          case peek_type
          when :star
            take
            args = ev(:args_add_star, args, arg(0))
          when :dstar
            take
            (assocs ||= []) << ev(:assoc_splat, arg(0))
          when :label
            label = take
            (assocs ||= []) << ev(:assoc_new, label, argument_start? ? arg(0) : nil)
          else
            value = arg(0)
            if value.is_a?(Ev) && value.name == :__label_string
              (assocs ||= []) << ev(:assoc_new, value.args[0], arg(0))
            elsif peek_type == :"=>"
              take
              skip_newlines
              (assocs ||= []) << ev(:assoc_new, value, arg(0))
            else
              args = ev(:args_add, args, value)
            end
          end
          skip_newlines
          break unless accept(:",")
        end
        args = ev(:args_add, args, ev(:bare_assoc_hash, assocs)) if assocs
        args
      end

      def array_literal
        take
        skip_newlines
        if peek_type == :"]"
          take
          return ev(:array, nil)
        end
        args = aref_args
        skip_newlines
        expect(:"]")
        ev(:array, args)
      end

      def hash_literal
        take
        skip_newlines
        assocs = []
        until peek_type == :"}"
          assocs << assoc
          skip_newlines
          break unless accept(:",")

          skip_newlines
        end
        skip_newlines
        expect(:"}")
        ev(:hash, assocs.empty? ? nil : ev(:assoclist_from_args, assocs))
      end

      def assoc
        case peek_type
        when :label
          label = take
          ev(:assoc_new, label, argument_start? ? arg(0) : nil)
        when :dstar
          take
          ev(:assoc_splat, argument_start? ? arg(0) : nil)
        else
          key = arg(0)
          if key.is_a?(Ev) && key.name == :__label_string
            return ev(:assoc_new, key.args[0], arg(0))
          end

          expect(:"=>")
          skip_newlines
          ev(:assoc_new, key, arg(0))
        end
      end

      # Blocks

      def brace_block
        open = take
        with_scope(true) do
          in_block do
            params = block_parameters
            stmts = statements
            expect(:"}")
            ev(open.type == :lbrace_arg ? :brace_block : :brace_block, params, stmts)
          end
        end
      end

      def do_block
        take
        with_scope(true) do
          in_block do
            params = block_parameters
            body = body_statement
            expect(:k_end)
            ev(:do_block, params, body)
          end
        end
      end

      def block_parameters
        skip_newlines
        return nil unless peek_type == :| || peek_type == :"||"

        @scope.ordinary_parameters = true

        if take.type == :"||"
          @scanner.command_start = true
          return ev(:block_var, empty_params, false)
        end
        if peek_type == :|
          take
          @scanner.command_start = true
          return ev(:block_var, empty_params, false)
        end
        params = peek_type == :";" ? empty_params : parameter_list(:|, block: true)
        locals = false
        if accept(:";")
          locals = []
          loop do
            name = expect(:ident)
            declare(name.text)
            locals << name
            break unless accept(:",")
          end
        end
        expect(:|)
        @scanner.command_start = true
        ev(:block_var, params, locals)
      end

      def empty_params = ev(:params, nil, nil, nil, nil, nil, nil, nil)

      # A parameter list up to `closer`, as the seven groups of a params
      # event.
      def parameter_list(closer, block: false)
        required = []
        optional = []
        rest = nil
        post = []
        keywords = []
        keyword_rest = nil
        block_arg = nil
        @scanner.in_argdef = true
        loop do
          case peek_type
          when :ident
            name = take
            declare(name.text)
            if peek_type == :"="
              take
              @scanner.in_argdef = false
              optional << [name, block ? postfix(primary(command: false)) : arg(0)]
              @scanner.in_argdef = true
            elsif rest || !optional.empty?
              post << name
            else
              required << name
            end
          when :lparen, :"(", :lparen_arg
            take
            inner = parameter_destructure
            expect(:")")
            (rest || !optional.empty? ? post : required) << ev(:mlhs_paren, inner)
          when :star, :*
            take
            name = peek_type == :ident ? take : nil
            declare(name.text) if name
            @scope.anonymous << :rest unless name || block
            rest = ev(:rest_param, name)
          when :dstar, :pow
            take
            if peek_type == :k_nil
              take
              keyword_rest = :nil
            else
              name = peek_type == :ident ? take : nil
              declare(name.text) if name
              @scope.anonymous << :keyword_rest unless name || block
              keyword_rest = ev(:kwrest_param, name)
            end
          when :amper, :&
            take
            name = peek_type == :ident ? take : nil
            declare(name.text) if name
            @scope.anonymous << :block unless name || block
            block_arg = ev(:blockarg, name)
          when :label
            label = take
            declare(label.text.chomp(":"))
            if argument_start? && ![:",", closer].include?(peek_type)
              @scanner.in_argdef = false
              keywords << [label, block ? postfix(primary(command: false)) : arg(0)]
              @scanner.in_argdef = true
            else
              keywords << [label, false]
            end
          when :bdot3
            take
            @scope.anonymous.push(:forward, :rest, :keyword_rest, :block)
            keyword_rest = ev(:args_forward)
          else
            unexpected
          end
          break unless peek_type == :","

          take
          skip_newlines
          if block && peek_type == closer && rest.nil?
            rest = ev(:excessed_comma)
            break
          end
        end
        @scanner.in_argdef = false
        ev(:params, list_or_nil(required), list_or_nil(optional), rest, list_or_nil(post),
           list_or_nil(keywords), keyword_rest, block_arg)
      end

      def list_or_nil(list) = list.empty? ? nil : list

      def parameter_destructure
        list = ev(:mlhs_new)
        post = nil
        loop do
          case peek_type
          when :star, :*
            take
            name = peek_type == :ident ? take : nil
            declare(name.text) if name
            list = ev(:mlhs_add_star, list, name)
            post = ev(:mlhs_new)
          when :lparen, :"(", :lparen_arg
            take
            inner = parameter_destructure
            expect(:")")
            list = ev(:mlhs_add, list, ev(:mlhs_paren, inner))
          else
            name = expect(:ident)
            declare(name.text)
            if post
              post = ev(:mlhs_add, post, name)
            else
              list = ev(:mlhs_add, list, name)
            end
          end
          break unless accept(:",")
        end
        post && post.name != :mlhs_new ? ev(:mlhs_add_post, list, post) : list
      end

      def lambda_literal
        take
        saved_lpar = @scanner.lpar_beg
        @scanner.lpar_beg = @scanner.paren_nest
        with_scope(true) do
          in_block do
            params = if [:"(", :lparen, :lparen_arg].include?(peek_type)
                       take
                       inner = peek_type == :")" ? empty_params : parameter_list(:")")
                       if accept(:";")
                         loop do
                           declare(expect(:ident).text)
                           break unless accept(:",")
                         end
                       end
                       expect(:")")
                       ev(:paren, inner)
                     elsif %i[lambeg k_do_lambda].include?(peek_type)
                       empty_params
                     else
                       parameter_list(:lambeg)
                     end
            if peek_type == :lambeg
              take
              stmts = statements
              expect(:"}")
              ev(:lambda, params, stmts)
            else
              expect(:k_do_lambda)
              body = body_statement
              expect(:k_end)
              ev(:lambda, params, body)
            end
          end
        end
      ensure
        @scanner.lpar_beg = saved_lpar
      end

      # Strings

      def string_parts(kind)
        list = ev(kind == :xstring ? :xstring_new : kind == :regexp ? :regexp_new : :string_content)
        add = kind == :xstring ? :xstring_add : kind == :regexp ? :regexp_add : :string_add
        loop do
          case peek_type
          when :string_content
            list = ev(add, list, take)
          when :string_dbeg
            list = ev(add, list, embedded_expression)
          when :string_dvar
            list = ev(add, list, embedded_variable)
          else
            return list
          end
        end
      end

      def embedded_expression
        take
        @scanner.cond_push(false)
        @scanner.cmdarg_push(false)
        stmts = statements
        expect(:string_dend)
        @scanner.cond_pop
        @scanner.cmdarg_pop
        ev(:string_embexpr, stmts)
      end

      def embedded_variable
        take
        token = take
        unless %i[ivar gvar cvar backref].include?(token.type)
          raise SyntaxFailure, "syntax error, unexpected '#{token.text}'"
        end
        ev(:string_dvar, token.type == :backref ? token : ev(:var_ref, token))
      end

      def dedented(list)
        width = @scanner.take_heredoc_indent
        return list unless width&.positive?

        contents = emit(list)
        @bridge.parser_event(:heredoc_dedent, contents, width)
        Dispatched.new(contents, list, width)
      end

      def single_string
        take
        list = string_parts(:string)
        if peek_type == :label_end
          take
          return ev(:__label_string, ev(:dyna_symbol, list))
        end
        expect(:string_end)
        ev(:string_literal, dedented(list))
      end

      def strings
        node = single_string
        return node if node.name == :__label_string

        while peek_type == :string_beg
          following = single_string
          raise SyntaxFailure, "syntax error, unexpected label" if following.name == :__label_string

          node = ev(:string_concat, node, following)
        end
        node
      end

      def xstring
        take
        list = string_parts(:xstring)
        expect(:string_end)
        ev(:xstring_literal, dedented(list))
      end

      def regexp
        take
        list = string_parts(:regexp)
        ending = expect(:regexp_end)
        ev(:regexp_literal, list, ending)
      end

      def words
        opener = take.type
        kind = { words_beg: :words, qwords_beg: :qwords, symbols_beg: :symbols, qsymbols_beg: :qsymbols }[opener]
        list = ev(:"#{kind}_new")
        take while peek_type == :words_sep
        until peek_type == :string_end
          if kind == :qwords || kind == :qsymbols
            list = ev(:"#{kind}_add", list, expect(:string_content))
          else
            word = ev(:word_new)
            while %i[string_content string_dbeg string_dvar].include?(peek_type)
              part = case peek_type
                     when :string_content then take
                     when :string_dbeg then embedded_expression
                     else embedded_variable
                     end
              word = ev(:word_add, word, part)
            end
            list = ev(:"#{kind}_add", list, word)
          end
          take while peek_type == :words_sep
        end
        take
        ev(:array, list)
      end

      def symbol
        opener = take
        if opener.text.length > 1
          list = string_parts(:string)
          expect(:string_end)
          @scanner.state = EXPR_END
          return ev(:dyna_symbol, list)
        end
        name = peek
        unless %i[ident const fid ivar gvar cvar backref].include?(name.type) || OPERATOR_NAMES.include?(name.type) ||
               name.type.to_s.start_with?("k_")
          unexpected
        end
        take
        @scanner.state = EXPR_END
        ev(:symbol_literal, ev(:symbol, name))
      end

      # Control structures

      # One term separates a condition from its body. Newlines after it are
      # not terms, and a further `;` is an empty statement in the body.
      def then_clause
        take if term?
        skip_newlines
        accept(:k_then)
      end

      def if_expression
        take
        condition = expr_value
        then_clause
        body = statements
        node = ev(:if, condition, body, if_tail)
        expect(:k_end)
        node
      end

      def if_tail
        case peek_type
        when :k_elsif
          take
          condition = expr_value
          then_clause
          body = statements
          ev(:elsif, condition, body, if_tail)
        when :k_else
          take
          ev(:else, statements)
        end
      end

      def unless_expression
        take
        condition = expr_value
        then_clause
        body = statements
        alternative = nil
        if peek_type == :k_else
          take
          alternative = ev(:else, statements)
        end
        expect(:k_end)
        ev(:unless, condition, body, alternative)
      end

      def loop_expression(kind)
        take
        @scanner.cond_push(true)
        condition = expr_value
        @scanner.cond_pop
        if peek_type == :k_do_cond
          take
        else
          unexpected unless term?
          take
          skip_newlines
        end
        body = in_loop { statements }
        expect(:k_end)
        ev(kind, condition, body)
      end

      def case_expression
        take
        subject = term? || %i[k_when k_in].include?(peek_type) ? nil : expr_value
        skip_terms
        if peek_type == :k_in
          clause = in_clause
          expect(:k_end)
          return ev(:case, subject, clause)
        end

        expect(:k_when)
        clause = when_clause
        expect(:k_end)
        ev(:case, subject, clause)
      end

      def when_clause
        args = ev(:args_new)
        loop do
          if peek_type == :star
            take
            args = ev(:args_add_star, args, arg(0))
          else
            args = ev(:args_add, args, arg(0))
          end
          break unless accept(:",")

          skip_newlines
        end
        then_clause
        body = statements
        following = case peek_type
                    when :k_when
                      take
                      when_clause
                    when :k_else
                      take
                      ev(:else, statements)
                    end
        ev(:when, args, body, following)
      end

      # Patterns

      def in_pattern_context
        @scanner.state = EXPR_BEG | EXPR_LABEL
        @scanner.command_start = false
        saved = @scanner.in_kwarg
        @scanner.in_kwarg = true
        yield
      ensure
        @scanner.in_kwarg = saved
      end

      def in_clause
        take
        pattern = in_pattern_context do
          top = pattern_top_body
          case peek_type
          when :k_if_mod
            take
            top = ev(:if_mod, expr_value, top)
          when :k_unless_mod
            take
            top = ev(:unless_mod, expr_value, top)
          end
          top
        end
        then_clause
        body = statements
        following = case peek_type
                    when :k_in then in_clause
                    when :k_else
                      take
                      ev(:else, statements)
                    end
        ev(:in, pattern, body, following)
      end

      def pattern_end?(closer = nil)
        type = peek_type
        return true if closer && type == closer

        term? || %i[k_then k_if_mod k_unless_mod eof k_and k_or].include?(type) || STATEMENT_ENDS.include?(type)
      end

      def pattern_top_body
        return hash_pattern(nil, nil) if hash_pattern_start?
        return array_pattern(nil, nil, []) if peek_type == :star

        first = pattern_expression
        return first unless peek_type == :","

        take
        return ev(:aryptn, nil, [first], nil, nil) if pattern_end?

        array_pattern(nil, nil, [first])
      end

      def hash_pattern_start? = %i[label dstar pow].include?(peek_type)

      def pattern_expression
        left = pattern_alternatives
        while peek_type == :"=>"
          take
          name = expect(:ident)
          bind_pattern_variable(name.text)
          left = ev(:binary, left, :"=>", ev(:var_field, name))
        end
        left
      end

      def pattern_alternatives
        saved = @pattern_bindings
        @pattern_bindings = []
        left = pattern_basic
        alternatives = false
        while peek_type == :|
          take
          alternatives = true
          left = ev(:binary, left, :|, pattern_basic)
        end
        if alternatives && (name = @pattern_bindings.find { |bound| !bound.start_with?("_") })
          report_error("illegal variable in alternative pattern (#{name})")
        end
        left
      ensure
        saved&.concat(@pattern_bindings)
        @pattern_bindings = saved
      end

      def bind_pattern_variable(name)
        declare(name)
        @pattern_bindings&.push(name)
      end

      def pattern_basic
        case peek_type
        when :ident
          name = take
          bind_pattern_variable(name.text)
          ev(:var_field, name)
        when :const, :colon3
          constant = pattern_constant
          case peek_type
          when :"("
            take
            skip_newlines
            node = pattern_items(constant, :")")
            skip_newlines
            expect(:")")
            node
          when :"["
            take
            skip_newlines
            node = pattern_items(constant, :"]")
            skip_newlines
            expect(:"]")
            node
          else
            pattern_range(constant)
          end
        when :lbrack
          take
          skip_newlines
          node = peek_type == :"]" ? ev(:aryptn, nil, nil, nil, nil) : array_pattern(nil, :"]", [])
          skip_newlines
          expect(:"]")
          node
        when :lbrace
          take
          skip_newlines
          node = hash_pattern(nil, :"}")
          skip_newlines
          expect(:"}")
          node
        when :lparen
          take
          skip_newlines
          node = pattern_expression
          skip_newlines
          expect(:")")
          node
        when :^
          take
          pinned
        when :bdot2, :bdot3
          type = take.type
          ev(type == :bdot2 ? :dot2 : :dot3, nil, pattern_primitive)
        else
          pattern_range(pattern_primitive)
        end
      end

      def pinned
        case peek_type
        when :ident, :ivar, :gvar, :cvar
          name = take
          if name.type == :ident && !local?(name.text)
            report_error("#{name.text}: no such local variable")
          end
          ev(:var_ref, name)
        when :lparen, :"(", :lparen_arg
          take
          skip_newlines
          value = expr_value
          skip_newlines
          expect(:")")
          ev(:begin, value)
        else
          unexpected
        end
      end

      def pattern_constant
        node = if peek_type == :colon3
                 take
                 ev(:top_const_ref, expect(:const))
               else
                 ev(:var_ref, expect(:const))
               end
        while peek_type == :colon2
          take
          node = ev(:const_path_ref, node, expect(:const))
        end
        node
      end

      def pattern_range(value)
        return value unless %i[dot2 dot3].include?(peek_type)

        type = take.type
        upper = pattern_end? || [:",", :|, :")", :"]", :"}", :"=>"].include?(peek_type) ? nil : pattern_primitive
        ev(type, value, upper)
      end

      def pattern_primitive
        return unary if %i[uminus_num uminus].include?(peek_type)

        type = peek_type
        unless %i[int float rational imaginary char string_beg xstring_beg regexp_beg words_beg qwords_beg
                  symbols_beg qsymbols_beg symbeg lambda].include?(type) || LITERAL_KEYWORDS.include?(type)
          unexpected
        end
        primary(command: false)
      end

      def pattern_items(constant, closer)
        return ev(:aryptn, constant, nil, nil, nil) if peek_type == closer
        return hash_pattern(constant, closer) if hash_pattern_start?

        array_pattern(constant, closer, [])
      end

      def array_pattern(constant, closer, pre)
        rest = nil
        post = []
        loop do
          break if pattern_end?(closer)

          if peek_type == :star
            take
            name = peek_type == :ident ? take : nil
            bind_pattern_variable(name.text) if name
            splat = ev(:var_field, name)
            if rest
              return finish_find_pattern(constant, closer, rest, post, splat)
            end

            rest = splat
          else
            (rest ? post : pre) << pattern_expression
          end
          break unless accept(:",")

          skip_newlines if closer
        end
        ev(:aryptn, constant, pre.empty? ? nil : pre, rest, post.empty? ? nil : post)
      end

      def finish_find_pattern(constant, closer, first, middle, last)
        skip_newlines if closer
        ev(:fndptn, constant, first, middle, last)
      end

      def hash_pattern(constant, closer)
        pairs = []
        rest = nil
        rest_marked = false
        loop do
          break if pattern_end?(closer)

          case peek_type
          when :label
            label = take
            if pattern_end?(closer) || peek_type == :","
              bind_pattern_variable(label.text.chomp(":"))
              pairs << [label, nil]
            else
              pairs << [label, pattern_expression]
            end
          when :string_beg
            take
            key = string_parts(:string)
            expect(:label_end)
            pairs << [key, pattern_end?(closer) || peek_type == :"," ? nil : pattern_expression]
          when :dstar, :pow
            take
            rest_marked = true
            if peek_type == :k_nil
              take
              rest = ev(:var_field, :nil)
            else
              name = peek_type == :ident ? take : nil
              bind_pattern_variable(name.text) if name
              rest = name ? ev(:var_field, name) : nil
            end
          else
            unexpected
          end
          break unless accept(:",")

          skip_newlines if closer
        end
        ev(:hshptn, constant, pairs.empty? && !rest_marked ? nil : pairs, rest)
      end

      def for_expression
        take
        variable = if peek_type == :star || peek_type == :lparen
                     list = mlhs_items(ev(:mlhs_new), true)
                     list
                   else
                     first = mlhs_target
                     if peek_type == :","
                       take
                       mlhs_items(ev(:mlhs_add, ev(:mlhs_new), first), true)
                     else
                       first
                     end
                   end
        expect(:k_in)
        @scanner.cond_push(true)
        iterable = expr_value
        @scanner.cond_pop
        if peek_type == :k_do_cond
          take
        else
          unexpected unless term?
          take
          skip_newlines
        end
        body = in_loop { statements }
        expect(:k_end)
        ev(:for, variable, iterable, body)
      end

      def begin_expression
        take
        @scanner.cmdarg_push(false)
        body = body_statement
        expect(:k_end)
        @scanner.cmdarg_pop
        ev(:begin, body)
      end

      def body_statement
        stmts = statements
        rescue_clause = nil
        else_clause = nil
        ensure_clause = nil
        rescue_clause = rescue_clauses if peek_type == :k_rescue
        if peek_type == :k_else
          take
          else_clause = statements
        end
        if peek_type == :k_ensure
          take
          ensure_clause = ev(:ensure, statements)
        end
        ev(:bodystmt, stmts, rescue_clause, else_clause, ensure_clause)
      end

      def rescue_clauses
        take
        exceptions = nil
        unless term? || %i[=> k_then].include?(peek_type)
          if peek_type == :star
            take
            exceptions = ev(:mrhs_add_star, ev(:mrhs_new), arg(0))
          else
            first = arg(0)
            if peek_type == :","
              take
              exceptions = multiple_rhs(ev(:args_add, ev(:args_new), first))
            else
              exceptions = [first]
            end
          end
        end
        variable = nil
        if accept(:"=>")
          variable = assignment_target(postfix(primary(command: false)))
        end
        then_clause
        saved = @in_rescue
        @in_rescue = true
        body = statements
        @in_rescue = saved
        following = peek_type == :k_rescue ? rescue_clauses : nil
        ev(:rescue, exceptions, variable, body, following)
      end

      def def_expression
        take
        singleton = nil
        operator = nil
        name = def_name_or_singleton
        if %i[. colon2 &.].include?(peek_type)
          singleton = singleton_node(name)
          operator = take
          @scanner.state = EXPR_FNAME
          name = method_name(fname: true)
          @scanner.state = EXPR_ENDFN | EXPR_LABEL
        end
        @scanner.in_argdef = true
        @bridge.compile_error("dynamic constant assignment") if false
        result = nil
        with_scope(false) do
          in_method_body do
            params = if peek_type == :"(" || peek_type == :lparen_arg || peek_type == :lparen
                       take
                       inner = peek_type == :")" ? empty_params : parameter_list(:")")
                       skip_newlines
                       expect(:")")
                       @scanner.state = EXPR_BEG
                       @scanner.command_start = true
                       ev(:paren, inner)
                     elsif peek_type == :"="
                       @scanner.in_argdef = false
                       empty_params
                     else
                       @scanner.state |= EXPR_LABEL
                       @scanner.in_kwarg = true
                       inner = term? ? empty_params : parameter_list(nil)
                       @scanner.in_kwarg = false
                       @scanner.in_argdef = false
                       unexpected unless term?
                       take
                       @scanner.state = EXPR_BEG
                       @scanner.command_start = true
                       inner
                     end
            @scanner.in_argdef = false
            if peek_type == :"="
              take
              skip_newlines
              value = arg(0, command: true)
              if peek_type == :k_rescue_mod
                take
                value = ev(:rescue_mod, value, arg(0))
              end
              body = ev(:bodystmt, value, nil, nil, nil)
            else
              body = body_statement
              expect(:k_end)
            end
            result = if singleton
                       ev(:defs, singleton, operator, name, params, body)
                     else
                       ev(:def, name, params, body)
                     end
          end
        end
        result
      end

      def def_name_or_singleton
        token = peek
        if token.type == :lparen || token.type == :"(" || token.type == :lparen_arg
          take
          value = expr(command: true)
          expect(:")")
          unexpected unless %i[. colon2 &.].include?(peek_type)
          return ev(:paren, value)
        end
        return take if %i[ivar gvar cvar].include?(token.type)

        method_name(fname: true)
      end

      def singleton_node(token)
        return token if token.is_a?(Ev)
        return ev(:var_ref, token) if LITERAL_KEYWORDS.include?(token.type) || %i[ivar gvar cvar const].include?(token.type)
        return ev(:var_ref, token) if token.type == :ident && local?(token.text)

        ev(:vcall, token)
      end

      def class_path
        if peek_type == :colon3
          take
          node = ev(:top_const_ref, expect(:const))
          while peek_type == :colon2
            take
            node = ev(:const_path_ref, node, expect(:const))
          end
          return node
        end
        node = primary(command: false)
        node = node.args[0] if node.is_a?(Ev) && node.name == :var_ref && peek_type != :colon2
        if node.is_a?(Token)
          raise SyntaxFailure, "class/module name must be CONSTANT" unless node.type == :const

          return ev(:const_ref, node)
        end
        while peek_type == :colon2
          take
          node = ev(:const_path_ref, node, expect(:const))
        end
        node
      end

      def class_expression
        take
        if peek_type == :<<
          take
          target = expr_value
          unexpected unless term?
          body = nil
          with_scope(false) do
            in_class_body(singleton: true) do
              skip_terms
              body = body_statement
            end
          end
          expect(:k_end)
          return ev(:sclass, target, body)
        end
        keyword = @taken.last
        path = class_path
        report_error("class definition in method body", keyword.index, @taken.size - 1) if @in_def
        superclass = nil
        if peek_type == :<
          take
          @scanner.state = EXPR_BEG
          @scanner.command_start = true
          superclass = expr_value
          unexpected unless term?
        end
        body = nil
        with_scope(false) do
          in_class_body do
            skip_terms if superclass
            body = body_statement
          end
        end
        expect(:k_end)
        ev(:class, path, superclass, body)
      end

      def module_expression
        keyword = take
        path = class_path
        report_error("module definition in method body", keyword.index, @taken.size - 1) if @in_def
        body = nil
        with_scope(false) { in_class_body { body = body_statement } }
        expect(:k_end)
        ev(:module, path, body)
      end

      def return_expression(command)
        keyword = take
        if @in_class && !@in_def && !@method_around_class && @block_depth.zero? && !@in_defined
          report_error("Invalid return in class/module body", keyword.index)
        end
        if command_follows?(command) || (command && argument_start?)
          return command_block(terminal(:return, jump_arguments))
        end

        ev(:return0)
      end

      def jump_expression(command)
        keyword = take
        name = keyword.text.to_sym
        entry = invalid_jump("Invalid #{name}", keyword) if @loop_depth.zero? && !@in_defined
        if command && argument_start?
          arguments = jump_arguments
          # The error names the jump with the value it carries.
          entry[2] = @taken.size - 1 if entry
          return command_block(terminal(name, arguments))
        end

        ev(name, ev(:args_new))
      end

      def jump_arguments
        @command_argument_depth += 1
        call_args(nil, command: true)
      ensure
        @command_argument_depth -= 1
      end

      def yield_expression(command)
        keyword = take
        report_error("Invalid yield", keyword.index) unless @in_def || @in_defined
        if peek_type == :"("
          take
          skip_newlines
          if peek_type == :")"
            take
            return ev(:yield, ev(:paren, ev(:args_new)))
          end
          args = call_args(:")", paren: true)
          skip_newlines
          expect(:")")
          return ev(:yield, ev(:paren, args))
        end
        if command_follows?(command)
          return command_block(terminal(:yield, command_args))
        end

        ev(:yield0)
      end

      def super_expression(command)
        take
        if peek_type == :"("
          return ev(:super, paren_args)
        end
        if command_follows?(command)
          return command_block(terminal(:super, command_args))
        end

        ev(:zsuper)
      end
    end
  end
end
