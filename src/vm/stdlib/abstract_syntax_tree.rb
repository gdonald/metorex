# RubyVM::AbstractSyntaxTree builds MRI's node tree for a program from the
# events the Ripper grammar parses it into. Each node's span comes from the
# tokens the grammar took for it.
require "ripper"

class RubyVM
  module AbstractSyntaxTree
    # Turns the grammar's events into nodes of the shape MRI's parser makes.
    class Builder
      Ev = Ripper::Engine::Ev
      Token = Ripper::Engine::Token

      # The tokens that close a construct, which belong to the event built
      # right after them.
      CLOSERS = %i[k_end rparen rbracket rbrace string_end tstring_end regexp_end label_end].freeze

      # The text an array literal can open with.
      ARRAY_OPENERS = ["[", "%w[", "%i[", "%W[", "%I["].freeze

      OPENING_BRACKETS = ["(", "[", "{"].freeze
      CLOSING_BRACKETS = [")", "]", "}"].freeze

      # The locals of a program, method, class or block body. A block reads
      # the locals of the bodies around it.
      Frame = Struct.new(:names, :block, :numbered, :it)

      def initialize(tokens, source = "")
        @tokens = tokens
        @source_lines = source.b.lines
        @frames = [Frame.new([], false, 0, false)]
        @index_at = {}
        @index_ending = {}
        @ranges = {}.compare_by_identity
        @argument_spans = {}.compare_by_identity
        @heredoc_pieces = {}.compare_by_identity
        @preexe = []
        @heredoc_ends = {}
        @pattern_spans = {}.compare_by_identity
        tokens.each do |token|
          @index_at[position_key(token.line, token.column)] ||= token.index
          @index_ending[position_key(*end_of(token.index))] = token.index unless token.type == :nl || token.text.to_s.empty?
        end
      end

      # The index of the token a node starts at.
      def index_of_start(converted)
        @index_at.fetch(position_key(converted.first_lineno, converted.first_column))
      end

      # One number for a line and column, which a hash looks up faster than
      # the pair.
      def position_key(line, column) = (line << 32) | column

      # The names MRI's parser gives the tokens metorex's scanner reads.
      TOKEN_NAMES = {
        ident: :tIDENTIFIER, const: :tCONSTANT, fid: :tFID, gvar: :tGVAR, ivar: :tIVAR, cvar: :tCVAR,
        label: :tLABEL, label_end: :tLABEL_END, int: :tINTEGER, float: :tFLOAT, rational: :tRATIONAL,
        imaginary: :tIMAGINARY, char: :tCHAR, string_beg: :tSTRING_BEG, string_end: :tSTRING_END,
        string_content: :tSTRING_CONTENT, string_dbeg: :tSTRING_DBEG, string_dend: :tSTRING_DEND,
        string_dvar: :tSTRING_DVAR, xstring_beg: :tXSTRING_BEG, regexp_beg: :tREGEXP_BEG,
        regexp_end: :tREGEXP_END, words_beg: :tWORDS_BEG, qwords_beg: :tQWORDS_BEG,
        symbols_beg: :tSYMBOLS_BEG, qsymbols_beg: :tQSYMBOLS_BEG, words_sep: :word_sep, symbeg: :tSYMBEG,
        colon2: :tCOLON2, colon3: :tCOLON3, dot2: :tDOT2, dot3: :tDOT3, bdot2: :tBDOT2, bdot3: :tBDOT3,
        star: :tSTAR, dstar: :tDSTAR, amper: :tAMPER, lambda: :tLAMBDA, lambeg: :tLAMBEG, lbrace: :tLBRACE,
        lbrace_arg: :tLBRACE_ARG, lbrack: :tLBRACK, lparen: :tLPAREN, lparen_arg: :tLPAREN_ARG,
        op_asgn: :tOP_ASGN, pow: :tPOW, uminus: :tUMINUS, uminus_num: :tUMINUS_NUM, uplus: :tUPLUS,
        aref: :tAREF, aset: :tASET, nl: :nl, "!=": :tNEQ, "!~": :tNMATCH, "&&": :tANDOP, "||": :tOROP,
        "<<": :tLSHFT, ">>": :tRSHFT, "==": :tEQ, "===": :tEQQ, "<=>": :tCMP, ">=": :tGEQ, "<=": :tLEQ,
        "=>": :tASSOC, "=~": :tMATCH, "&.": :tANDDOT, "(": :'"("', ")": :'")"', "[": :'"["', "]": :'"]"',
        "{": :'"{"', "}": :'"}"', k_do_lambda: :keyword_do_LAMBDA, "k_defined?": :keyword_defined,
        k___FILE__: :keyword__FILE__, k___LINE__: :keyword__LINE__, k___ENCODING__: :keyword__ENCODING__
      }.freeze

      # The names of the tokens the grammar skips.
      SKIPPED_TOKEN_NAMES = {
        on_sp: :tSP, on_comment: :tCOMMENT, on_ignored_nl: :tIGNORED_NL, on_embdoc_beg: :tEMBDOC_BEG,
        on_embdoc: :tEMBDOC, on_embdoc_end: :tEMBDOC_END
      }.freeze

      def token_name(token)
        return TOKEN_NAMES[token.type] if TOKEN_NAMES.key?(token.type)

        name = token.type.to_s
        if token.type == :backref
          token.text.match?(/\A\$\d+\z/) ? :tNTH_REF : :tBACK_REF
        elsif name.start_with?("k_") && name.end_with?("_mod")
          :"modifier_#{name[2...-4]}"
        elsif name.start_with?("k_")
          :"keyword_#{name[2..]}"
        else
          token.type
        end
      end

      # Every token of the program the way MRI lists them with
      # `keep_tokens`: in the order its lexer reads them, with the spaces,
      # comments and blank lines the grammar skips, and the text of each.
      def all_tokens(source)
        lexed = Ripper.lex(source)
        ending = lexed.index { |_, event, _, _| event == :on___end__ }
        lexed = lexed.first(ending) if ending
        text_at = {}
        newlines = {}
        lexed.each do |(line, column), event, text, _|
          text_at[position_key(line, column)] = text
          newlines[position_key(line, column)] = true if event == :on_nl
        end
        # A newline that ends a comment is part of the comment's text.
        kept = @tokens.reject { |token| token.type == :nl && !newlines.key?(position_key(token.line, token.column)) }
        # A heredoc's terminator is listed with the newline that ends it.
        listed = kept.map do |token|
          name = token_name(token)
          text = token.text
          if @heredoc_ends.key?(token.index)
            name = :tHEREDOC_END
            text = text_at.fetch(position_key(token.line, token.column), text)
          elsif %i[string_beg xstring_beg].include?(token.type) && token.text.start_with?("<<")
            name = :tHEREDOC_BEG
          end
          [name, text, token.line, token.column]
        end
        taken = {}
        @tokens.each { |token| taken[position_key(token.line, token.column)] = true }
        # Each skipped token goes before the first grammar token written after
        # it, outside heredoc bodies, which the lexer reads out of line order.
        body = heredoc_body_indexes
        anchors = kept.each_index.reject { |index| body.key?(kept[index].index) }.sort_by { |index| [kept[index].line, kept[index].column] }
        before = Hash.new { |hash, key| hash[key] = [] }
        trailing = []
        lexed.each do |(line, column), event, text, _|
          next unless SKIPPED_TOKEN_NAMES.key?(event) && !taken.key?(position_key(line, column))

          entry = [SKIPPED_TOKEN_NAMES[event], text, line, column]
          found = anchors.bsearch { |index| ([kept[index].line, kept[index].column] <=> [line, column]) > 0 }
          found ? before[found] << entry : trailing << entry
        end
        listed = listed.each_with_index.flat_map { |entry, index| before.fetch(index, []) + [entry] } + trailing
        # MRI hands each token's text over as bytes.
        listed.each_with_index.map do |(name, text, line, column), index|
          [index, name, text.b, [line, column, *token_end(line, column, text)]]
        end
      end

      # Where text written from `line` and `column` ends. A newline that
      # ends it ends on its last line.
      def token_end(line, column, text)
        trailing = text.end_with?("\n") ? 1 : 0
        lines = text.chomp("\n").split("\n", -1)
        return [line, column + text.bytesize] if lines.size <= 1

        [line + lines.size - 1, lines.last.bytesize + trailing]
      end

      # Gives each node of a finished tree its id and the parse it belongs
      # to. A node's id is its place in the tree with children counted
      # first.
      def finish(root, tree)
        count = 0
        visit = lambda do |value|
          case value
          when Node
            value.children.each { |child| visit.call(child) }
            value.instance_variable_set(:@node_id, count)
            value.instance_variable_set(:@tree, tree)
            count += 1
          when Array then value.each { |item| visit.call(item) }
          end
        end
        visit.call(root)
        root
      end

      # Where a program whose last token is at index `last` ends. A comment
      # after that token takes in the newline that would end the program,
      # and MRI places that newline where it stopped reading: at the last of
      # the comment lines that follow, after the comment when another kind
      # of line follows, or at the comment when the file ends on its line.
      def program_end(last)
        finish = end_of(last)
        return finish if @tokens[last].type == :";"

        line, column = finish
        text = @source_lines[line - 1].to_s
        rest = text.byteslice(column..).to_s
        return finish unless rest.match?(/\A[ \t\f\v\r]*#/)

        comment = [line, column + rest.index("#")]
        return comment if @source_lines[line].nil?

        following = line
        following += 1 while @source_lines[following].to_s.match?(/\A[ \t\f\v\r]*#/)
        return [line, text.bytesize] if following == line

        [following, @source_lines[following - 1].index("#")]
      end

      # The SCOPE node of a whole program.
      def program(tree)
        body = heredoc_body_indexes
        written = @tokens.each_index.reject { |index| @tokens[index].type == :nl || body.include?(index) }
        span = written.empty? ? [1, 0, 1, 0] : [1, 0, *program_end(written.last)]
        # An empty program is a BEGIN with nothing in it.
        body = statements(tree.args[0]) || node(:BEGIN, [nil], [1, 0, 1, 0])
        unless @preexe.empty?
          main = body.type == :BLOCK ? body.children : [body]
          body = node(:BLOCK, @preexe + main, between(@preexe.first, main.last))
        end
        node(:SCOPE, [@frames.last.names, nil, body], span)
      end

      # The spans of the keywords and operators MRI records for a node of
      # its type, in the order `Node#locations` lists them after the node's
      # own span, or nil for a type that records none.
      def recorded_locations(node)
        own = owned_tokens(node)
        find = ->(*types) { own.find { |token| types.include?(token.type) } }
        last = ->(*types) { own.reverse_each.find { |token| types.include?(token.type) } }
        opening = ->(token) { token if token && [token.line, token.column] == [node.first_lineno, node.first_column] }
        spans = case node.type
                when :ALIAS, :VALIAS then [find.(:k_alias)]
                when :AND then [find.(:"&&", :k_and)]
                when :OR then [find.(:"||", :k_or, :|)]
                when :BLOCK_PASS then [block_pass_operator(node)]
                when :BREAK then [find.(:k_break)]
                when :NEXT then [find.(:k_next)]
                when :REDO then [find.(:k_redo)]
                when :RETURN then [find.(:k_return)]
                when :CASE, :CASE2, :CASE3 then [find.(:k_case), last.(:k_end)]
                when :CLASS then [find.(:k_class), find.(:<), last.(:k_end)]
                when :SCLASS then [find.(:k_class), find.(:<<), last.(:k_end)]
                when :MODULE then [find.(:k_module), last.(:k_end)]
                when :COLON2 then [find.(:colon2), own.last]
                when :COLON3 then [find.(:colon3), own.last]
                when :DEFINED then [find.(:"k_defined?")]
                when :DOT2, :DOT3, :FLIP2, :FLIP3 then [find.(:dot2, :dot3, :bdot2, :bdot3)]
                when :EVSTR then [find.(:string_dbeg, :string_dvar), find.(:string_dend)]
                when :FOR then [find.(:k_for), find.(:k_in), find.(:k_do_cond), last.(:k_end)]
                when :LAMBDA then lambda_tokens(node)
                when :IF, :UNLESS then conditional_tokens(node, own, opening)
                when :IN then pattern_branch_tokens(node, own)
                when :WHEN then [find.(:k_when), then_span(node, own, find.(:k_when))]
                when :OP_ASGN1 then [nil, find.(:"[", :lbrack), find.(:"]"), find.(:op_asgn)]
                when :OP_ASGN2 then [find.(:".", :"&.", :colon2), find.(:ident, :const, :fid), find.(:op_asgn)]
                when :POSTEXE then [find.(:k_END), find.(:"{"), last.(:"}")]
                when :REGX then regexp_tokens(own)
                when :SPLAT then [find.(:star, :*)]
                when :SUPER then [find.(:k_super), find.(:"(", :lparen_arg), last.(:")")]
                when :YIELD then [find.(:k_yield), find.(:"(", :lparen_arg), last.(:")")]
                when :UNDEF then [find.(:k_undef)]
                when :WHILE then [find.(:k_while, :k_while_mod), opening.(find.(:k_while)) && last.(:k_end)]
                when :UNTIL then [find.(:k_until, :k_until_mod), opening.(find.(:k_until)) && last.(:k_end)]
                else return nil
                end
        spans.map { |held| held.is_a?(Token) ? span_of_tokens(held.index, held.index) : held }
      end

      private

      # Whether the span of `inner` lies inside that of `outer` and is not
      # all of it.
      def within_span?(inner, outer)
        first = [inner.first_lineno, inner.first_column]
        last = [inner.last_lineno, inner.last_column]
        (first <=> [outer.first_lineno, outer.first_column]) >= 0 &&
          (last <=> [outer.last_lineno, outer.last_column]) <= 0 &&
          [first, last] != [[outer.first_lineno, outer.first_column], [outer.last_lineno, outer.last_column]]
      end

      # The tokens written within a node and outside each of its children.
      # A child spanning all of the node or more, such as the scope of a
      # class body, gives way to its own children.
      def owned_tokens(node)
        inside = ->(token, first, last) do
          finish = end_of(token.index)
          ([token.line, token.column] <=> first) >= 0 && (finish <=> last) <= 0
        end
        children = []
        gather = lambda do |value|
          case value
          when Node
            if within_span?(value, node)
              children << value
            else
              value.children.each { |child| gather.call(child) }
            end
          when Array then value.each { |item| gather.call(item) }
          end
        end
        node.children.each { |child| gather.call(child) }
        first = [node.first_lineno, node.first_column]
        last = [node.last_lineno, node.last_column]
        @tokens.select do |token|
          next false unless inside.(token, first, last)

          children.none? do |child|
            inside.(token, [child.first_lineno, child.first_column], [child.last_lineno, child.last_column])
          end
        end
      end

      # The `if`, `elsif` or `unless`, the `then` and the `end` of a
      # conditional. A ternary records its `:` as the `then`, and an `elsif`
      # the `end` that closes the whole conditional after it.
      def conditional_tokens(node, own, opening)
        if (mark = own.find { |token| token.type == :"?" })
          return [nil, own.find { |token| token.type == :":" && token.index > mark.index }, nil]
        end

        keyword = own.find { |token| %i[k_if k_elsif k_if_mod k_unless k_unless_mod].include?(token.type) }
        return [keyword, nil, nil] unless opening.(keyword)

        closing = if keyword&.type == :k_elsif
                    following = end_index_of(node)
                    following += 1 while following && %i[nl ;].include?(@tokens[following]&.type)
                    following && @tokens[following]&.type == :k_end ? @tokens[following] : nil
                  else
                    own.reverse_each.find { |token| token.type == :k_end }
                  end
        [keyword, then_span(node, own, keyword), closing]
      end

      # The `in` and `then` of a pattern branch. A one-line pattern match
      # written `value in pattern` or `value => pattern` records the `in`
      # or the `=>` written before the pattern.
      def pattern_branch_tokens(node, own)
        keyword = own.find { |token| token.type == :k_in }
        return [keyword, then_span(node, own, keyword), nil] if keyword

        before = @tokens.reverse_each.find do |token|
          token.type != :nl && (end_of(token.index) <=> [node.first_lineno, node.first_column]) <= 0
        end
        case before&.type
        when :k_in then [before, nil, nil]
        when :"=>" then [nil, nil, before]
        else [nil, nil, nil]
        end
      end

      # The `&` of a block argument, which an anonymous one is written as
      # alone.
      def block_pass_operator(node)
        block = node.children.last
        @tokens.reverse_each.find do |token|
          %i[amper &].include?(token.type) &&
            ([token.line, token.column] <=> [block.first_lineno, block.first_column]) <= 0 &&
            ([token.line, token.column] <=> [node.first_lineno, node.first_column]) >= 0
        end
      end

      # The index of the first token after a node.
      def end_index_of(node)
        @tokens.index { |token| ([token.line, token.column] <=> [node.last_lineno, node.last_column]) >= 0 }
      end

      # Where the `then` of a branch is: the newline or `;` that ends its
      # condition, through a `then` keyword that follows it. A newline is
      # recorded without width, where the line's text ends.
      def then_span(node, own, keyword)
        return nil if keyword.nil?

        condition = node.children[0]
        reached = end_of(keyword.index)
        if condition.is_a?(Node) && within_span?(condition, node)
          reached = [reached, [condition.last_lineno, condition.last_column]].max
        end
        term = nil
        depth = 0
        own.each do |token|
          next if token.index <= keyword.index

          depth = [depth + nesting(token), 0].max
          next if ([token.line, token.column] <=> reached).negative?
          break term = token if depth.zero? && %i[nl ; k_then].include?(token.type)

          reached = [reached, end_of(token.index)].max
        end
        line, column = reached
        text = @source_lines[line - 1].to_s
        gap = text.byteslice(column..).to_s[/\A[ \t]*/].to_s
        newline_at = column + gap.bytesize
        # MRI places the newline after a comment past the line's end.
        newline_at = text.bytesize if text.getbyte(newline_at) == "#".ord
        newline = text.getbyte(newline_at).nil? || text.getbyte(newline_at) == 10
        first = if newline && (term.nil? || term.line > line || term.type == :nl)
                  [line, newline_at, line, newline_at]
                elsif term && term.type != :nl
                  span_of_tokens(term.index, term.index)
                end
        return nil if first.nil?
        return first if term&.type == :k_then && first == span_of_tokens(term.index, term.index)

        following = @tokens.find do |token|
          token.type != :nl && ([token.line, token.column] <=> first[2, 2]) >= 0
        end
        return first unless following&.type == :k_then && own.include?(following)

        [*first[0, 2], *end_of(following.index)]
      end

      # A regexp's opening, its text, and its closing. An empty regexp's
      # text has no width.
      def regexp_tokens(own)
        opening = own.find { |token| token.type == :regexp_beg }
        closing = own.reverse_each.find { |token| token.type == :regexp_end }
        content = own.find { |token| token.type == :string_content }
        content ||= opening && [*end_of(opening.index), *end_of(opening.index)]
        [opening, content, closing]
      end

      # A lambda's `->`, the `{` or `do` that opens its body and the `}` or
      # `end` that closes it. Its body's scope covers the braces.
      def lambda_tokens(node)
        first = [node.first_lineno, node.first_column]
        last = [node.last_lineno, node.last_column]
        written = @tokens.select do |token|
          ([token.line, token.column] <=> first) >= 0 && (end_of(token.index) <=> last) <= 0
        end
        [written.first, written.find { |token| %i[lambeg k_do_lambda].include?(token.type) }, written.last]
      end

      # How a token changes the bracket depth, counting the `#{` that opens
      # an interpolation.
      def nesting(token)
        return 1 if OPENING_BRACKETS.include?(token.text) || token.type == :string_dbeg
        return -1 if CLOSING_BRACKETS.include?(token.text)

        0
      end

      # The indexes of the tokens in heredoc bodies, which come after the
      # line the heredoc is written on.
      def heredoc_body_indexes
        indexes = {}
        @tokens.each_with_index do |token, index|
          next unless %i[string_beg xstring_beg].include?(token.type) && token.text.start_with?("<<")

          depth = 0
          (index + 1...@tokens.size).each do |at|
            type = @tokens[at].type
            depth += 1 if %i[string_beg xstring_beg].include?(type)
            if type == :string_end
              break if depth.zero?

              depth -= 1
            end
            indexes[at] = true
          end
          closing = (index + 1...@tokens.size).find { |at| @tokens[at].type == :string_end && !indexes.key?(at) }
          if closing
            indexes[closing] = true
            @heredoc_ends[closing] = true
          end
        end
        indexes
      end

      def node(type, children, span)
        Node.new(type, children, *span)
      end

      # Where token `index` starts, as a line and a column.
      def start_of(index)
        token = @tokens[index]
        [token.line, token.column]
      end

      # Where token `index` ends, as a line and a column. A token that runs
      # over several lines ends on the last of them.
      def end_of(index)
        token = @tokens[index]
        text = token.text.to_s
        # A newline that ends a token ends on the token's last line.
        trailing = text.end_with?("\n") && token.type != :nl ? 1 : 0
        lines = text.chomp("\n").split("\n", -1) if trailing == 1
        lines ||= text.split("\n", -1)
        return [token.line, token.column + text.bytesize] if lines.size <= 1

        [token.line + lines.size - 1, lines.last.bytesize + trailing]
      end

      def span_of_tokens(first, last)
        [*start_of(first), *end_of(last)]
      end

      # The first and last token an event's arguments name, at any depth.
      def token_range(value)
        case value
        when Token
          [value.index, value.index]
        when Ev
          return @ranges[value] if @ranges.key?(value)

          ranges = value.args.map { |arg| token_range(arg) }.compact
          @ranges[value] = ranges.empty? ? nil : [ranges.map(&:first).min, ranges.map(&:last).max]
        when Array
          ranges = value.map { |item| token_range(item) }.compact
          return nil if ranges.empty?

          [ranges.map(&:first).min, ranges.map(&:last).max]
        end
      end

      # A statement list: nothing, the one statement, or a BLOCK of them.
      def statements(list)
        written = flatten_statements(list)
        nodes = written.map { |statement| unwrap_begin(convert(statement)) }.compact
        return empty_body(list) if nodes.empty?

        # A body that opens with an empty statement, after a `;`, keeps it.
        leading_empty = written.first.is_a?(Ev) && written.first.name == :void_stmt
        nodes.unshift(empty_body(list)) if leading_empty
        return nodes.first if nodes.size == 1

        first = nodes.first
        last = nodes.last
        node(:BLOCK, nodes, [first.first_lineno, first.first_column, last.last_lineno, last.last_column])
      end

      # A body with nothing in it, which MRI writes as a BEGIN of no width
      # at the end of the last token before the body.
      def empty_body(list)
        opened = list
        opened = opened.args[0] while opened.is_a?(Ev) && opened.name == :stmts_add
        index = opened.is_a?(Ev) ? opened.last : -1
        return nil if index.negative?

        # The body begins where what opened it ends, or where the newline
        # that opened it starts.
        line, column = @tokens[index].type == :nl ? start_of(index) : end_of(index)
        node(:BEGIN, [nil], [line, column, line, column])
      end

      # A `begin` written as a statement is only its body.
      def unwrap_begin(converted)
        converted = converted.children[0] while converted && converted.type == :BEGIN && converted.children[0]
        converted
      end

      def flatten_statements(list)
        return [] if list.nil?
        return [list] unless list.is_a?(Ev)

        case list.name
        when :stmts_new then []
        when :stmts_add then flatten_statements(list.args[0]) + [list.args[1]]
        else [list]
        end
      end

      def convert(value)
        case value
        when Token then convert_token(value)
        when Ev then convert_event(value)
        end
      end

      def convert_token(token)
        span = span_of_tokens(token.index, token.index)
        case token.type
        when :int then node(:INTEGER, [Integer(token.text.delete("_"))], span)
        when :float then node(:FLOAT, [Float(token.text.delete("_"))], span)
        when :rational then node(:RATIONAL, [rational(token.text.delete("_").chomp("r"))], span)
        when :imaginary then node(:IMAGINARY, [imaginary(token.text)], span)
        when :char then node(:STR, [cooked(token.text[1..], :double)], span)
        when :backref, :gvar, :ivar, :cvar then on_var_ref(Ev.new(:var_ref, [token], false, token.index))
        else unknown(token.type, span)
        end
      end

      # A rational literal's value. One written with a radix prefix is a
      # whole number, which `Rational` does not read.
      def rational(number)
        number.match?(/\A0[xbod]/i) ? Rational(Integer(number)) : Rational(number)
      end

      def imaginary(text)
        number = text.delete("_").chomp("i")
        if number.end_with?("r")
          Complex(0, rational(number.chomp("r")))
        elsif number.match?(/\A0[xbod]/i)
          Complex(0, Integer(number))
        elsif number.include?(".") || number.match?(/e/i)
          Complex(0, Float(number))
        else
          Complex(0, Integer(number))
        end
      end

      def convert_event(event)
        handler = "on_#{event.name}"
        return __send__(handler, event) if respond_to?(handler, true)

        range = token_range(event)
        unknown(event.name, range ? span_of_tokens(*range) : [0, 0, 0, 0])
      end

      # A form the converter does not know yet. It is named after the event
      # so a dump shows what is missing.
      def unknown(name, span)
        node(:"UNKNOWN_#{name}", [], span)
      end

      # A span from where one node starts to where another ends.
      def between(first, last)
        [first.first_lineno, first.first_column, last.last_lineno, last.last_column]
      end

      # A span from where token `index` starts to where a node ends.
      def from_token(index, last)
        [*start_of(index), last.last_lineno, last.last_column]
      end

      # A span from where a node starts to where token `index` ends.
      def to_token(first, index)
        [first.first_lineno, first.first_column, *end_of(index)]
      end

      # The span a value's tokens cover.
      def span_of(value)
        span_of_tokens(*token_range(value))
      end

      # The index of the first token a value names.
      def first_index(value)
        token_range(value).first
      end

      # Note a local the body being built assigns.
      def declare(name)
        @frames.last.names << name unless local_name?(name)
      end

      # Whether a local is visible here: in this body, or in a body around a
      # block.
      def local_name?(name)
        @frames.reverse_each do |frame|
          return true if frame.names.include?(name)
          break unless frame.block
        end
        false
      end

      def in_block? = @frames.last.block

      # A local read, which MRI names a DVAR inside a block.
      def local_read(name, span) = node(in_block? ? :DVAR : :LVAR, [name], span)

      def local_write_type = in_block? ? :DASGN : :LASGN

      # An anonymous `*`, `**` or `&` passed on, written at token `index`,
      # which is an LVAR inside a block too.
      def anonymous_read(name, index) = node(:LVAR, [name], span_of_tokens(index, index))

      # Builds a body's nodes with a frame of its own for its locals.
      def within(block)
        @frames.push(Frame.new([], block, 0, false))
        yield @frames.last
      ensure
        @frames.pop
      end

      # Variables, constants and the names a call is written with.

      def on_var_ref(event)
        token = event.args[0]
        span = span_of_tokens(token.index, token.index)
        case token.type
        when :k_nil then node(:NIL, [], span)
        when :k_true then node(:TRUE, [], span)
        when :k_false then node(:FALSE, [], span)
        when :k_self then node(:SELF, [], span)
        when :k___FILE__ then node(:FILE, [""], span)
        when :k___LINE__ then node(:LINE, [token.line], span)
        when :ivar then node(:IVAR, [token.text.to_sym], span)
        when :gvar then node(:GVAR, [token.text.to_sym], span)
        when :backref
          type = token.text.match?(/\A\$\d+\z/) ? :NTH_REF : :BACK_REF
          node(type, [token.text.to_sym], span)
        when :cvar then node(:CVAR, [token.text.to_sym], span)
        when :const then node(:CONST, [token.text.to_sym], span)
        else
          name = token.text.to_sym
          numbered = name.match?(/\A_[1-9]\z/) && in_block? && !local_name?(name)
          @frames.last.numbered = [@frames.last.numbered, name.to_s[1].to_i].max if numbered
          local_read(name, span)
        end
      end

      def on_vcall(event)
        token = event.args[0]
        span = span_of_tokens(token.index, token.index)
        # `it` in a block that names no parameters is the block's argument.
        if token.text == "it" && in_block? && @frames.last.it != :named
          @frames.last.it = true
          return node(:DVAR, [:"<it>"], span)
        end
        node(:VCALL, [token.text.to_sym], span)
      end

      def on_const_path_ref(event)
        scope = convert(event.args[0])
        name = event.args[1]
        node(:COLON2, [scope, name.text.to_sym], to_token(scope, name.index))
      end

      def on_top_const_ref(event)
        name = event.args[0]
        node(:COLON3, [name.text.to_sym], span_of_tokens(name.index - 1, name.index))
      end

      # Calls.

      def on_call(event)
        receiver = convert(event.args[0])
        operator, name = event.args[1], event.args[2]
        type = operator.is_a?(Token) && operator.text == "&." ? :QCALL : :CALL
        # `a.()` names no method, and ends at its dot until its parentheses
        # are added.
        last = name.is_a?(Token) ? name.index : operator.index
        node(type, [receiver, method_name(name), nil], to_token(receiver, last))
      end

      def method_name(name)
        return :call if name == :call

        name.text.to_sym
      end

      def on_fcall(event)
        token = event.args[0]
        node(:FCALL, [token.text.to_sym, nil], span_of_tokens(token.index, token.index))
      end

      # A call given arguments in parentheses: the call it is, with those
      # arguments, ending at the closing parenthesis.
      def on_method_add_arg(event)
        call = convert(event.args[0])
        paren = event.args[1]
        arguments = if forwarding?(paren)
          forwarded_arguments(paren)
        elsif paren.is_a?(Ev) && paren.name == :arg_paren
          arguments_node(paren.args[0])
        else
          arguments_node(paren)
        end
        children = call.children.dup
        children[-1] = arguments
        node(call.type, children, to_token(call, paren.last))
      end

      def forwarding?(paren)
        return false unless paren.is_a?(Ev) && paren.name == :arg_paren

        inner = paren.args[0]
        inner.is_a?(Ev) && (inner.name == :args_forward || (inner.name == :args_add && inner.args[1].is_a?(Ev) && inner.args[1].name == :args_forward))
      end

      # The arguments `(...)` passes on: the rest, the keywords and the block
      # the method was given, each read at the `...`, after any written
      # before it.
      def forwarded_arguments(paren)
        inner = paren.args[0]
        close = paren.last
        open = matching_open(close, "(", ")")
        written = inner.name == :args_add ? combine_arguments(argument_elements(inner.args[0])) : nil
        dots = inner.name == :args_add ? inner.args[1].last : close - 1
        at_dots = span_of_tokens(dots, dots)
        whole = span_of_tokens(open, close)
        rest = anonymous_read(:*, dots)
        splat = written ? node(:ARGSCAT, [written, rest], whole) : node(:SPLAT, [rest], at_dots)
        keywords = node(:HASH, [node(:LIST, [nil, anonymous_read(:**, dots), nil], at_dots)], at_dots)
        pushed = node(:ARGSPUSH, [splat, keywords], whole)
        node(:BLOCK_PASS, [pushed, anonymous_read(:&, dots)], whole)
      end

      def on_command(event)
        name, arguments = event.args
        argument = arguments_node(arguments)
        node(:FCALL, [name.text.to_sym, argument], from_token(name.index, argument))
      end

      def on_command_call(event)
        receiver = convert(event.args[0])
        operator, name, arguments = event.args[1], event.args[2], event.args[3]
        type = operator.is_a?(Token) && operator.text == "&." ? :QCALL : :CALL
        # A call on a command's `do` block may name no arguments, or put
        # them in parentheses.
        return node(type, [receiver, method_name(name), nil], to_token(receiver, name.index)) if arguments.nil?
        if arguments.is_a?(Ev) && arguments.name == :arg_paren
          return node(type, [receiver, method_name(name), arguments_node(arguments.args[0])], to_token(receiver, arguments.last))
        end

        argument = arguments_node(arguments)
        node(type, [receiver, method_name(name), argument], between(receiver, argument))
      end

      def on_aref(event)
        receiver = convert(event.args[0])
        argument = arguments_node(event.args[1])
        node(:CALL, [receiver, :[], argument], to_token(receiver, event.last))
      end

      def on_binary(event)
        left = convert(event.args[0])
        operator = event.args[1]
        right = convert(event.args[2])
        logic = { "&&": :AND, and: :AND, "||": :OR, or: :OR }[operator]
        if logic
          # A chain of the same operator is one node, its operands its
          # children.
          operands = left.type == logic ? [*left.children, right] : [left, right]
          return node(logic, operands, between(left, right))
        end
        return match(left, right) if operator == :=~

        list = node(:LIST, [right, nil], between(right, right))
        node(:OPCALL, [left, operator, list], between(left, right))
      end

      # `=~` with a regexp literal on the left is a MATCH2, which assigns the
      # groups a plain regexp names. One on the right with interpolation is a
      # MATCH3. Anything else is a call of `=~`.
      def match(left, right)
        span = between(left, right)
        regexp = left.type == :ONCE ? left.children[0] : left
        return node(:MATCH2, [left, right], span) if regexp.type == :DREGX
        if regexp.type == :REGX
          captures = named_captures(regexp.children[0], span)
          return node(:MATCH2, [left, right, captures].compact, span)
        end
        return node(:MATCH3, [right, left], span) if (right.type == :ONCE ? right.children[0] : right).type == :DREGX

        node(:CALL, [left, :=~, node(:LIST, [right, nil], read_span(right))], span)
      end

      def named_captures(regexp, span)
        names = regexp.names.select { |name| name.match?(/\A[a-z_][A-Za-z0-9_]*\z/) }
        return nil if names.empty?

        assignments = names.map do |name|
          symbol = name.to_sym
          declare(symbol)
          node(local_write_type, [symbol, node(:SYM, [symbol], span)], span)
        end
        node(:BLOCK, assignments, span)
      end

      def on_unary(event)
        operator, operand = event.args
        if operator == :-@ && operand.is_a?(Token) && %i[int float rational imaginary].include?(operand.type)
          literal = convert_token(operand)
          span = [*start_of(operand.index - 1), *end_of(operand.index)]
          return node(literal.type, [-literal.children[0]], span)
        end
        value = convert(operand)
        # `!` and `not` read their operand as a condition.
        value = cond(value, read_span(value)) if operator == :not || operator == :!
        if operator == :not
          # `not` may wrap its operand in parentheses, which the node spans.
          start = opener_before(index_of_start(value), ["not"])
          last = index_of_start(value) - 1 > start ? closing_paren_after(value) : nil
          return node(:OPCALL, [value, :!, nil], last ? span_of_tokens(start, last) : from_token(start, value))
        end
        node(:OPCALL, [value, operator, nil], from_token(index_of_start(value) - 1, value))
      end

      # Arguments.

      # The elements an argument list names, in order, each as
      # [:plain, value], [:splat, value, star_index] or
      # [:pairs, elements, span].
      def argument_elements(list)
        return [] if list.nil? || list == false
        return [[:plain, list]] unless list.is_a?(Ev)

        case list.name
        when :args_new, :mrhs_new then []
        when :args_add, :mrhs_add
          before = argument_elements(list.args[0])
          item = list.args[1]
          if item.is_a?(Ev) && item.name == :bare_assoc_hash
            before + [[:pairs, item.args[0]]]
          else
            before + [[:plain, item]]
          end
        when :args_add_star, :mrhs_add_star
          # An anonymous `*` is the last token its event took.
          value = list.args[1]
          argument_elements(list.args[0]) + [[:splat, value, list.last]]
        when :mrhs_new_from_args then argument_elements(list.args[0])
        when :args_add_block then argument_elements(list.args[0])
        else [[:plain, list]]
        end
      end

      # The node for an argument list, the way MRI's parser shapes it.
      def arguments_node(list, outer = nil)
        return nil if list.nil? || list == false

        block = list.is_a?(Ev) && list.name == :args_add_block ? list.args[1] : false
        elements = argument_elements(list)
        built = combine_arguments(elements, outer)
        return built if block == false

        # A nil block is an anonymous `&`, the last token of the list.
        passed = block.nil? ? anonymous_read(:&, list.last) : convert(block)
        ampersand = block.nil? ? list.last : index_of_start(passed) - 1
        start = built ? [built.first_lineno, built.first_column] : start_of(ampersand)
        # Keywords pushed after a splat reach to the block argument.
        if built&.type == :ARGSPUSH && built.children[1].type == :HASH
          built = node(:ARGSPUSH, built.children, between(built, passed))
        end
        node(:BLOCK_PASS, [built, passed], [*start, passed.last_lineno, passed.last_column])
      end

      def combine_arguments(elements, outer = nil)
        return nil if elements.empty?

        built = nil
        plain = []
        elements.each do |element|
          kind, value, anonymous_star = element
          case kind
          when :plain, :pairs
            item = kind == :pairs ? pairs_hash(value) : convert(value)
            if built
              built = node(:ARGSPUSH, [built, item], between(built, item))
            else
              plain << item
            end
          when :splat
            item = value.nil? ? anonymous_read(:*, anonymous_star) : convert(value)
            star = value.nil? ? anonymous_star : index_of_start(item) - 1
            if built
              built = node(:ARGSCAT, [built, item], between(built, item))
            elsif plain.empty?
              built = node(:SPLAT, [item], from_token(star, item))
            else
              list = node(:LIST, plain + [nil], between(plain.first, plain.last))
              built = node(:ARGSCAT, [list, item], between(list, item))
            end
          end
        end
        built ||= node(:LIST, plain + [nil], [*argument_span(plain.first).first(2), *argument_span(plain.last).last(2)])
        return built if outer.nil?

        node(built.type, built.children, outer)
      end

      # Where an argument is written, which for one in a command's
      # parentheses includes them.
      def argument_span(item) = @argument_spans[item] || read_span(item)

      # A hash written without braces, among a call's arguments.
      def pairs_hash(pairs)
        list = pair_list(pairs)
        node(:HASH, [list], [list.first_lineno, list.first_column, list.last_lineno, list.last_column])
      end

      # The keys and values of a hash, flattened into a LIST, with nil
      # standing for the key of a `**` splat.
      def pair_list(pairs)
        items = []
        pairs.each do |pair|
          case pair.name
          when :assoc_new
            key, value = pair.args
            items << (key.is_a?(Token) && key.type == :label ? label_symbol(key) : convert(key))
            items << (value.nil? ? shorthand_value(key) : convert(value))
          when :assoc_splat
            items << nil
            items << (pair.args[0] ? convert(pair.args[0]) : anonymous_read(:**, pair.last))
          end
        end
        first = items.compact.first
        # A nil first key is a `**` splat, which starts at its stars, before
        # the value it spreads. An anonymous one is read at its stars.
        start = items.first.nil? && pairs.first.args[0] ? start_of(index_of_start(first) - 1) : [first.first_lineno, first.first_column]
        last = items.last
        node(:LIST, items + [nil], [*start, last.last_lineno, last.last_column])
      end


      # The value `name:` stands for: the local of that name, or a call.
      def shorthand_value(label)
        name = label.text.chomp(":").to_sym
        span = span_of_tokens(label.index, label.index)
        local_name?(name) ? local_read(name, span) : node(:VCALL, [name], span)
      end

      def label_symbol(token)
        node(:SYM, [token.text.chomp(":").to_sym], span_of_tokens(token.index, token.index))
      end

      # The index of the nearest token before `index` written as one of
      # `texts`.
      def opener_before(index, texts)
        index -= 1 while index.positive? && !texts.include?(@tokens[index].text)
        index
      end

      def on_array(event)
        return word_list(event) if word_list?(event.args[0])
        return node(:ZLIST, [], span_of_tokens(event.last - 1, event.last)) if event.args[0].nil?

        built = combine_arguments(argument_elements(event.args[0]))
        opener = matching_open(event.last, "[", "]")
        node(built.type, built.children, span_of_tokens(opener, event.last))
      end

      def on_hash(event)
        pairs = event.args[0]
        span = span_of_tokens(matching_open(event.last, "{", "}"), event.last)
        return node(:HASH, [nil], span) if pairs.nil?

        node(:HASH, [pair_list(pairs.args[0])], span)
      end

      # Assignments.

      def on_assign(event)
        target, value_event = event.args
        # The target is a local from where it is written, ahead of its value.
        declare(target.args[0].text.to_sym) if target.name == :var_field && target.args[0].type == :ident
        value = convert(value_event)
        assignment(target, value)
      end

      # The node assigning `value` to a target, spanning both.
      def assignment(target, value)
        start = first_index(target)
        span = if value
          from_token(start, value)
        elsif target.name == :aref_field
          span_of_tokens(start, target.last)
        else
          span_of(target)
        end
        case target.name
        when :var_field
          token = target.args[0]
          name = token.text.to_sym
          case token.type
          when :ivar then node(:IASGN, [name, value], span)
          when :gvar then node(:GASGN, [name, value], span)
          when :cvar then node(:CVASGN, [name, value], span)
          when :const then node(:CDECL, [name, value], span)
          else
            declare(name)
            node(local_write_type, [name, value], span)
          end
        when :const_path_field
          path = on_const_path_ref(target)
          node(:CDECL, [path, path.children[1], value], span)
        when :top_const_field
          path = on_top_const_ref(target)
          node(:CDECL, [path, path.children[0], value], span)
        when :field
          receiver = convert(target.args[0])
          operator, name = target.args[1], target.args[2]
          written = operator.is_a?(Token) && operator.text == "&." ? name.text.to_sym : :"#{name.text}="
          list = value && node(:LIST, [value, nil], between(value, value))
          node(:ATTRASGN, [receiver, written, list], span)
        when :aref_field
          receiver = convert(target.args[0])
          elements = argument_elements(target.args[1])
          elements += [[:plain_node, value]] if value
          items = elements.map { |kind, item| kind == :plain_node ? item : convert(item) }
          list = items.empty? ? nil : node(:LIST, items + [nil], between(items.first, items.last))
          node(:ATTRASGN, [receiver, :[]=, list], span)
        else
          unknown(:"assign_#{target.name}", span)
        end
      end

      def on_opassign(event)
        target, operator_token, value_event = event.args
        value = convert(value_event)
        operator = operator_token.text.chomp("=").to_sym
        span = from_token(first_index(target), value)
        case target.name
        when :var_field
          token = target.args[0]
          read = on_var_ref(Ev.new(:var_ref, [token], false, token.index))
          read = local_read(token.text.to_sym, read_span(read)) if token.type == :ident
          if %i[|| &&].include?(operator)
            type = operator == :"||" ? :OP_ASGN_OR : :OP_ASGN_AND
            assigned = assignment(target, value)
            node(type, [read, operator, node(assigned.type, assigned.children, span)], span)
          else
            assigned = assignment(target, nil)
            call = node(:CALL, [read, operator, node(:LIST, [value, nil], between(value, value))], span)
            node(assigned.type, [assigned.children[0], call], span)
          end
        when :field
          receiver = convert(target.args[0])
          safe = target.args[1].is_a?(Token) && target.args[1].text == "&."
          node(:OP_ASGN2, [receiver, safe, target.args[2].text.to_sym, operator, value], span)
        when :aref_field
          receiver = convert(target.args[0])
          index = arguments_node(target.args[1])
          node(:OP_ASGN1, [receiver, operator, index, value], span)
        when :const_path_field, :top_const_field
          path = target.name == :const_path_field ? on_const_path_ref(target) : on_top_const_ref(target)
          node(:OP_CDECL, [path, operator, value], span)
        else
          unknown(:"opassign_#{target.name}", span)
        end
      end

      def read_span(read)
        [read.first_lineno, read.first_column, read.last_lineno, read.last_column]
      end

      def on_massign(event)
        targets, value_event = event.args
        value = massign_value(value_event)
        left = multiple_targets(targets)
        node(:MASGN, [value, *left], [*start_of(first_target_index(targets)), value.last_lineno, value.last_column])
      end

      # The index of the first token of a multiple assignment's targets,
      # which is the star when the first of them is a splat.
      def first_target_index(targets)
        innermost = targets
        innermost = innermost.args[0] while innermost.is_a?(Ev) && %i[mlhs_add mlhs_add_star mlhs_add_post].include?(innermost.name) && !(innermost.args[0].is_a?(Ev) && innermost.args[0].name == :mlhs_new)
        index = first_index(targets)
        return index unless innermost.is_a?(Ev)

        # A first target written as a splat or in parentheses starts at
        # the star or the parenthesis.
        first = innermost.args[1]
        index -= 1 if innermost.name == :mlhs_add_star
        index -= 1 if first.is_a?(Ev) && first.name == :mlhs_paren
        index
      end

      # The value a multiple assignment takes apart: a LIST of the values
      # written, a splat, or the one value.
      # Several values on the right of an assignment.
      def on_mrhs_add(event) = combine_arguments(argument_elements(event))

      def on_mrhs_add_star(event) = combine_arguments(argument_elements(event))

      def on_mrhs_new_from_args(event) = combine_arguments(argument_elements(event))

      def massign_value(value_event)
        if value_event.is_a?(Ev) && %i[mrhs_add mrhs_add_star mrhs_new_from_args].include?(value_event.name)
          return combine_arguments(argument_elements(value_event))
        end

        convert(value_event)
      end

      # The targets of a multiple assignment, as the LIST of those before a
      # splat and the splat's own target.
      def multiple_targets(targets)
        before = []
        rest = nil
        collect = lambda do |list|
          case list.name
          when :mlhs_new then nil
          when :mlhs_add
            collect.call(list.args[0])
            before << list.args[1]
          when :mlhs_add_star
            collect.call(list.args[0])
            rest = list.args[1]
          when :mlhs_add_post
            collect.call(list.args[0])
          end
        end
        collect.call(targets)
        nodes = before.map { |target| target_node(target) }
        list = nodes.empty? ? nil : node(:LIST, nodes + [nil], [*start_of(first_target_index(targets)), nodes.last.last_lineno, nodes.last.last_column])
        [list, rest && target_node(rest)]
      end

      def target_node(target)
        if target.is_a?(Ev) && target.name == :mlhs_paren
          left = multiple_targets(target.args[0])
          inner = [left[0], left[1]].compact
          return node(:MASGN, [nil, *left], between(inner.first, inner.last))
        end
        assignment(target, nil)
      end

      # Control flow.

      # A node read as a condition: a regexp matches `$_`, and a range is a
      # flip-flop whose Integer ends compare with `$.`. `span` is where the
      # statement holding the condition is written. `and`, `or`, `(...)`
      # and `begin` pass the condition on to what they hold.
      def cond(converted, span, top = true)
        case converted&.type
        when :BEGIN
          body = converted.children[0]
          body ? node(:BEGIN, [cond(body, span, top)], read_span(converted)) : converted
        when :ONCE then cond(converted.children[0], span, top)
        when :REGX then node(:MATCH, converted.children, read_span(converted))
        when :DREGX then node(:MATCH2, [converted, node(:GVAR, [:$_], span)], span)
        when :BLOCK
          children = converted.children.dup
          children[-1] = cond(children[-1], span, top && children.size == 1)
          node(:BLOCK, children, read_span(converted))
        when :AND, :OR
          node(converted.type, converted.children.map { |child| cond(child, span) }, read_span(converted))
        when :DOT2, :DOT3
          return converted unless top

          ends = converted.children.map { |child| flip_flop_end(child, span) }
          node(converted.type == :DOT2 ? :FLIP2 : :FLIP3, ends, read_span(converted))
        else converted
        end
      end

      def flip_flop_end(converted, span)
        return cond(converted, span) unless converted.type == :INTEGER

        node(:CALL, [converted, :==, node(:LIST, [node(:GVAR, [:"$."], span), nil], span)], span)
      end

      def condition(event) = convert(event)

      # The body that follows a branch, an `else` clause or nothing.
      def branch(clause)
        return nil if clause.nil?
        return statements(clause.args[0]) if clause.is_a?(Ev) && clause.name == :else

        convert(clause)
      end

      def on_if(event) = conditional(:IF, event)

      def on_unless(event) = conditional(:UNLESS, event)

      def conditional(type, event)
        test = condition(event.args[0])
        body = statements(event.args[1])
        rest = branch(event.args[2])
        span = span_of_tokens(index_of_start(test) - 1, closing_end(event))
        node(type, [cond(test, span), body, rest], span)
      end

      # The index of the `)` that follows a node.
      def closing_paren_after(converted)
        index = index_of_start(converted)
        index += 1 until @tokens[index].text == ")"
        index
      end

      # The `end` that closes an event built before its `end` was taken.
      def closing_end(event)
        index = event.last
        index += 1 until @tokens[index].type == :k_end
        index
      end

      def on_elsif(event)
        test = condition(event.args[0])
        body = statements(event.args[1])
        rest = branch(event.args[2])
        last = rest || body
        span = from_token(index_of_start(test) - 1, last)
        node(:IF, [cond(test, span), body, rest], span)
      end

      def on_ifop(event)
        test = condition(event.args[0])
        positive = convert(event.args[1])
        negative = convert(event.args[2])
        span = between(test, negative)
        node(:IF, [cond(test, span), positive, negative], span)
      end

      def on_if_mod(event) = modifier(:IF, event)

      def on_unless_mod(event) = modifier(:UNLESS, event)

      def modifier(type, event)
        test = condition(event.args[0])
        body = convert(event.args[1])
        span = between(body, test)
        node(type, [cond(test, span), body, nil], span)
      end

      def on_while(event) = loop_node(:WHILE, event)

      def on_until(event) = loop_node(:UNTIL, event)

      def loop_node(type, event)
        test = condition(event.args[0])
        test = cond(test, read_span(test))
        body = statements(event.args[1])
        node(type, [test, body, true], span_of_tokens(index_of_start(test) - 1, event.last))
      end

      def on_while_mod(event) = loop_modifier(:WHILE, event)

      def on_until_mod(event) = loop_modifier(:UNTIL, event)

      # `body while test`, where a `begin` body runs before the test is
      # first read.
      def loop_modifier(type, event)
        test = condition(event.args[0])
        test = cond(test, read_span(test))
        body_event = event.args[1]
        if body_event.is_a?(Ev) && body_event.name == :begin
          body = unwrap_begin(convert(body_event))
          start = opener_before(token_range(body_event)&.first || event.last, ["begin"])
          return node(type, [test, body, false], from_token(start, test))
        end
        body = convert(body_event)
        node(type, [test, body, true], between(body, test))
      end

      def on_case(event)
        return pattern_case(event) if event.args[1].is_a?(Ev) && event.args[1].name == :in

        subject = event.args[0] && convert(event.args[0])
        clauses = convert(event.args[1])
        start = subject ? index_of_start(subject) - 1 : opener_before(first_index(event.args[1]), ["case"])
        type = subject ? :CASE : :CASE2
        node(type, [subject, clauses], span_of_tokens(start, event.last))
      end

      def on_when(event)
        values = combine_arguments(argument_elements(event.args[0]))
        body = statements(event.args[1])
        rest = branch(event.args[2])
        last = rest || body
        start = opener_before(first_index(event.args[0]), ["when"])
        node(:WHEN, [values, body, rest], from_token(start, last))
      end

      # A `begin` written where a value goes, which keeps a BEGIN around its
      # body from `begin` to `end`.
      def on_begin(event)
        body = body_statement(event.args[0], event)
        range = token_range(event.args[0])
        start = opener_before(range ? range.first : event.last, ["begin"])
        node(:BEGIN, [body], span_of_tokens(start, closing_end(event)))
      end

      # A body with its `rescue`, `else` and `ensure` clauses.
      # A method's empty body is nil, where other bodies are an empty BEGIN.
      def body_statement(bodystmt, opener = nil, empty: :begin)
        statements_event, rescue_event, else_event, ensure_event = bodystmt.args
        body = statements(statements_event)
        if body.nil? || (body.type == :BEGIN && body.children == [nil] && rescue_event.nil? && ensure_event.nil?)
          body ||= empty_body(statements_event)
        end
        anchor = body
        body = nil if empty == :nil && empty_begin?(body)
        if rescue_event
          clauses = convert(rescue_event)
          otherwise = else_event && statements(else_event.is_a?(Ev) && else_event.name == :else ? else_event.args[0] : else_event)
          body = node(:RESCUE, [body, clauses, otherwise], between(anchor, otherwise || clauses))
          anchor = body
        end
        if ensure_event
          ensured = statements(ensure_event.args[0])
          body = node(:ENSURE, [body, ensured], [anchor.first_lineno, anchor.first_column, *through_semicolons(ensured)])
        end
        body
      end

      def empty_begin?(converted)
        converted&.type == :BEGIN && converted.children == [nil] && converted.first_column == converted.last_column && converted.first_lineno == converted.last_lineno
      end

      # Where a clause ends: after its last node, and after any `;` that
      # follow it.
      def through_semicolons(last)
        position = [last.last_lineno, last.last_column]
        index = @tokens.index { |token| ([token.line, token.column] <=> position) >= 0 }
        while index && %i[; nl].include?(@tokens[index].type)
          position = end_of(index) if @tokens[index].type == :";"
          index += 1
        end
        position
      end

      def on_rescue(event)
        exceptions, variable, body_event, rest_event = event.args
        listed = rescued_classes(exceptions)
        assigned = nil
        if variable
          target_start = first_index(variable) - 1
          read = node(:ERRINFO, [], span_of_tokens(target_start, token_range(variable).last))
          assigned = assignment(variable, read)
          assigned = node(assigned.type, assigned.children, read_span(read))
        end
        body = statements(body_event)
        rest = rest_event && convert(rest_event)
        anchor = token_range(exceptions) || token_range(variable) || token_range(body_event)
        start = opener_before(anchor ? anchor.first : event.last, ["rescue"])
        finish = rest ? [rest.last_lineno, rest.last_column] : through_semicolons(body)
        node(:RESBODY, [listed, assigned, body, rest], [*start_of(start), *finish])
      end

      def rescued_classes(exceptions)
        return nil if exceptions.nil?
        return combine_arguments(exceptions.map { |item| [:plain, item] }) if exceptions.is_a?(Array)

        combine_arguments(argument_elements(exceptions))
      end

      def on_rescue_mod(event)
        body = convert(event.args[0])
        fallback = convert(event.args[1])
        start = index_of_start(fallback) - 1
        clause = node(:RESBODY, [nil, nil, fallback, nil], from_token(start, fallback))
        node(:RESCUE, [body, clause, nil], between(body, fallback))
      end

      def on_break(event) = jump(:BREAK, event)

      def on_next(event) = jump(:NEXT, event)

      def on_return(event) = jump(:RETURN, event)

      # `break`, `next` or `return`, with the value it carries.
      def jump(type, event)
        value = jump_value(event.args[0])
        return node(type, [nil], span_of_tokens(event.last, event.last)) if value.nil?

        node(type, [value], from_token(index_of_start(value) - 1, value))
      end

      def jump_value(arguments)
        elements = argument_elements(arguments)
        return nil if elements.empty?
        return convert(elements.first[1]) if elements.size == 1 && elements.first[0] == :plain

        arguments_node(arguments)
      end

      def on_return0(event) = node(:RETURN, [nil], span_of_tokens(event.last, event.last))

      def on_redo(event) = node(:REDO, [], span_of_tokens(event.last, event.last))

      def on_retry(event) = node(:RETRY, [], span_of_tokens(event.last, event.last))

      def on_for(event)
        targets, iterated, body_event = event.args
        iteration = convert(iterated)
        start = (targets.is_a?(Ev) && targets.name == :var_field ? first_index(targets) : first_target_index(targets)) - 1
        span = span_of_tokens(start, event.last)
        parameters = for_parameters(targets, index_of_start(iteration) - 2)
        body = statements(body_event)
        node(:FOR, [iteration, node(:SCOPE, [[nil], parameters, body], span)], span)
      end

      # The parameters a `for` loop's hidden block takes: a local of its own
      # as the one parameter, or a multiple assignment from it.
      def for_parameters(targets, last_target)
        if targets.is_a?(Ev) && targets.name == :var_field && targets.args[0].type == :ident
          token = targets.args[0]
          declare(token.text.to_sym)
          span = span_of_tokens(token.index, token.index)
          assigned = node(local_write_type, [token.text.to_sym, node(:DVAR, [nil], span)], span)
          return node(:ARGS, [1, assigned, nil, nil, 0, nil, nil, nil, nil, nil], span)
        end
        several = targets.is_a?(Ev) && targets.name.start_with?("mlhs")
        if several
          left = multiple_targets(targets)
          span = span_of_tokens(first_target_index(targets), last_target)
        else
          assigned = assignment(targets, nil)
          left = [node(:LIST, [assigned, nil], read_span(assigned)), nil]
          span = read_span(assigned)
        end
        read = node(:DVAR, [nil], span)
        read = node(:FOR_MASGN, [read], span) if several
        assigned = node(:MASGN, [read, *left], span)
        node(:ARGS, [0, assigned, nil, nil, 0, nil, nil, nil, nil, nil], span)
      end

      def on_dot2(event) = range(:DOT2, event)

      def on_dot3(event) = range(:DOT3, event)

      # A range, where a missing end is a nil of no width beside the
      # operator.
      def range(type, event)
        low_event, high_event = event.args
        low = low_event && convert(low_event)
        high = high_event && convert(high_event)
        if low.nil?
          operator = index_of_start(high) - 1
          line, column = start_of(operator)
          low = node(:NIL, [], [line, column, line, column])
          return node(type, [low, high], from_token(operator, high))
        end
        if high.nil?
          operator = token_range(low_event).last + 1
          line, column = end_of(operator)
          high = node(:NIL, [], [line, column, line, column])
          return node(type, [low, high], to_token(low, operator))
        end
        node(type, [low, high], between(low, high))
      end

      def on_void_stmt(_event) = nil

      # Statements in parentheses: a BLOCK around what they are.
      def on_paren(event)
        opener = matching_open(event.last, "(", ")")
        body = if opener + 1 == event.last
          line, column = end_of(opener)
          node(:BEGIN, [nil], [line, column, line, column])
        else
          statements(event.args[0])
        end
        # A command's argument in parentheses is what they hold, and the
        # argument list spans the parentheses.
        if @tokens[opener].type == :lparen_arg
          @argument_spans[body] = span_of_tokens(opener, event.last)
          return body
        end
        node(:BLOCK, [body], span_of_tokens(opener, event.last))
      end



      # Definitions and blocks.

      # The index of the token a node ends at.
      def index_of_end(converted)
        @index_ending.fetch(position_key(converted.last_lineno, converted.last_column))
      end

      # The index of the `open` that `close`, at index `index`, closes.
      def matching_open(index, open, close)
        depth = 0
        index.downto(0) do |at|
          text = @tokens[at].text
          depth += 1 if text == close
          # An interpolation's `#{` opens a brace too.
          depth -= 1 if text == open || (open == "{" && @tokens[at].type == :string_dbeg)
          return at if depth.zero?
        end
      end

      # The tokens a parameter list is written with between its delimiters,
      # up to any `;` that starts its block locals.
      def parameter_tokens(first, last)
        depth = 0
        (first..last).each do |at|
          depth += nesting(@tokens[at])
          if depth.zero? && @tokens[at].text == ";"
            last = at - 1
            break
          end
        end
        first += 1 while first <= last && @tokens[first].type == :nl
        last -= 1 while last >= first && @tokens[last].type == :nl
        first > last ? nil : [first, last]
      end

      # A parameter written in parentheses to take an argument apart, with
      # each name in it as an assignment target.
      def destructured_targets(mlhs)
        case mlhs
        when Token then Ev.new(:var_field, [mlhs], false, mlhs.index)
        when Ev
          args = mlhs.args.map { |arg| destructured_targets(arg) }
          Ev.new(mlhs.name, args, mlhs.terminal, mlhs.last)
        else mlhs
        end
      end

      def destructured?(item) = item.is_a?(Ev) && item.name == :mlhs_paren

      # The MASGN that takes apart the arguments given to destructured
      # parameters.
      def destructuring(items)
        assignments = items.select { |item| destructured?(item) }.map do |item|
          left = multiple_targets(destructured_targets(item.args[0]))
          # A rest parameter spans its star.
          left[1] = node(left[1].type, left[1].children, from_token(index_of_start(left[1]) - 1, left[1])) if left[1]
          inner = left.compact
          span = between(inner.first, inner.last)
          line, column = span
          read = node(in_block? ? :DVAR : :LVAR, [nil], [line, column, line, column])
          node(:MASGN, [read, *left], span)
        end
        return nil if assignments.empty?
        return assignments.first if assignments.size == 1

        node(:BLOCK, assignments, between(assignments.first, assignments.last))
      end

      # A chain of OPT_ARG or KW_ARG nodes, each spanning from its own
      # parameter to the last of them.
      def parameter_chain(type, assignments)
        assignments.reverse.reduce(nil) do |rest, assigned|
          node(type, [assigned, rest], between(assigned, rest || assigned))
        end
      end

      # The name a `*`, `**` or `&` parameter declares, and the token it is
      # written with.
      def anonymous_or_named(event, anonymous)
        token = event.args[0]
        token ? [token.text.to_sym, token.index] : [anonymous, nil]
      end

      # The ARGS node for a parameter list written over tokens `range`,
      # declaring each parameter in the order MRI's local table lists them.
      def parameters(params, range, locals = [])
        pre, optional, rest, post, keywords, keyword_rest, block = params.args
        pre ||= []
        optional ||= []
        post ||= []
        keywords ||= []
        forwarding = keyword_rest.is_a?(Ev) && keyword_rest.name == :args_forward
        frame = @frames.last
        named = ->(item) { destructured?(item) ? nil : item.text.to_sym }
        pre.each { |item| frame.names << named.(item) }
        optional.each { |name, _| frame.names << name.text.to_sym }
        rest_name = case rest
                    when nil then forwarding ? :* : nil
                    when Ev
                      rest.name == :excessed_comma ? :NODE_SPECIAL_EXCESSIVE_COMMA : anonymous_or_named(rest, :*)[0]
                    end
        frame.names << rest_name if rest_name && rest_name != :NODE_SPECIAL_EXCESSIVE_COMMA
        post.each { |item| frame.names << named.(item) }
        keywords.each { |label, _| frame.names << label.text.chomp(":").to_sym }
        frame.names << nil unless keywords.empty?
        keyword_rest_name = if forwarding
          :**
        elsif keyword_rest.is_a?(Ev)
          anonymous_or_named(keyword_rest, :**)[0]
        end
        frame.names << keyword_rest_name if keyword_rest_name
        block_name = forwarding ? :& : block && anonymous_or_named(block, :&)[0]
        frame.names << block_name if block_name
        frame.names << :"..." if forwarding
        locals.each { |token| frame.names << token.text.to_sym }

        pre_init = destructuring(pre)
        post_init = destructuring(post)
        assign = local_write_type
        optional_nodes = optional.map do |name, value_event|
          value = convert(value_event)
          node(assign, [name.text.to_sym, value], from_token(name.index, value))
        end
        keyword_nodes = keywords.map do |label, value_event|
          name = label.text.chomp(":").to_sym
          if value_event
            value = convert(value_event)
            node(assign, [name, value], from_token(label.index, value))
          else
            node(assign, [name, :NODE_SPECIAL_REQUIRED_KEYWORD], span_of_tokens(label.index, label.index))
          end
        end
        keyword_chain = parameter_chain(:KW_ARG, keyword_nodes)
        keyword_rest_node = if keyword_rest == :nil
          false
        elsif forwarding
          dots = (range[0]..range[1]).find { |at| @tokens[at].text == "..." }
          node(:DVAR, [:**], span_of_tokens(dots, dots))
        elsif keyword_rest.is_a?(Ev)
          name, index = anonymous_or_named(keyword_rest, :**)
          star = index ? index - 1 : (range[0]..range[1]).find { |at| @tokens[at].text == "**" }
          node(:DVAR, [name], span_of_tokens(star, index || star))
        elsif keyword_chain
          node(:DVAR, [nil], read_span(keyword_chain))
        end
        keyword_chain = false if keyword_rest == :nil && keyword_chain.nil?
        first_post = post.empty? ? nil : named.(post.first)
        children = [pre.size, pre_init, parameter_chain(:OPT_ARG, optional_nodes), first_post, post.size, post_init,
                    rest_name, keyword_chain, keyword_rest_node, block_name]
        node(:ARGS, children, span_of_tokens(*range))
      end

      # An ARGS node for no parameters, of no width at `position`.
      def no_parameters(position)
        line, column = position
        node(:ARGS, [0, nil, nil, nil, 0, nil, nil, nil, nil, nil], [line, column, line, column])
      end

      # The ARGS of a method: its parameter list, which may be in
      # parentheses, or nothing.
      def method_parameters(params_event, name)
        if params_event.is_a?(Ev) && params_event.name == :paren
          close = params_event.last
          open = matching_open(close, "(", ")")
          range = parameter_tokens(open + 1, close - 1)
          return parameters(params_event.args[0], range) if range

          return node(:ARGS, no_parameters(start_of(open)).children, span_of_tokens(open, open))
        end
        range = token_range(params_event)
        return parameters(params_event, range) if range

        no_parameters(end_of(name.index))
      end

      def on_def(event)
        name, params_event, bodystmt = event.args
        opener = opener_before(name.index, ["def"])
        within(false) do |frame|
          args = method_parameters(params_event, name)
          # An endless method written without parameters spans them from
          # `def` to its name.
          args = node(:ARGS, args.children, span_of_tokens(opener, name.index)) if endless?(event) && unwritten?(params_event)
          body = body_statement(bodystmt, empty: :nil)
          span = span_of_tokens(opener, event.last)
          scope = node(:SCOPE, [frame.names, args, body], span)
          node(:DEFN, [name.text.to_sym, scope], span)
        end
      end

      def on_defs(event)
        receiver_event, _operator, name, params_event, bodystmt = event.args
        receiver = convert(receiver_event)
        opener = opener_before(index_of_start(receiver), ["def"])
        within(false) do |frame|
          args = method_parameters(params_event, name)
          args = node(:ARGS, args.children, span_of_tokens(opener, name.index)) if endless?(event) && unwritten?(params_event)
          body = body_statement(bodystmt, empty: :nil)
          span = span_of_tokens(opener, event.last)
          scope = node(:SCOPE, [frame.names, args, body], span)
          node(:DEFS, [receiver, name.text.to_sym, scope], span)
        end
      end

      # Whether a method is written as `def name = value`.
      def endless?(event)
        @tokens[event.last].type != :k_end
      end

      # Whether a method is written with no parameters and no parentheses.
      def unwritten?(params_event)
        !(params_event.is_a?(Ev) && params_event.name == :paren) && token_range(params_event).nil?
      end

      # The node naming the class or module a definition opens.
      def constant_path(path)
        case path.name
        when :const_ref
          token = path.args[0]
          node(:COLON2, [nil, token.text.to_sym], span_of_tokens(token.index, token.index))
        else convert(path)
        end
      end

      def on_class(event)
        path_event, superclass_event, bodystmt = event.args
        path = constant_path(path_event)
        superclass = superclass_event && convert(superclass_event)
        opener = opener_before(index_of_start(path), ["class"])
        span = span_of_tokens(opener, event.last)
        within(false) do |frame|
          body = body_statement(bodystmt)
          node(:CLASS, [path, superclass, node(:SCOPE, [frame.names, nil, body], span)], span)
        end
      end

      def on_module(event)
        path_event, bodystmt = event.args
        path = constant_path(path_event)
        opener = opener_before(index_of_start(path), ["module"])
        span = span_of_tokens(opener, event.last)
        within(false) do |frame|
          body = body_statement(bodystmt)
          node(:MODULE, [path, node(:SCOPE, [frame.names, nil, body], span)], span)
        end
      end

      def on_sclass(event)
        target_event, bodystmt = event.args
        target = convert(target_event)
        opener = opener_before(index_of_start(target), ["class"])
        span = span_of_tokens(opener, event.last)
        within(false) do |frame|
          body = body_statement(bodystmt)
          node(:SCLASS, [target, node(:SCOPE, [frame.names, nil, body], span)], span)
        end
      end

      # A call given a block: an ITER of the call and the block's SCOPE.
      def on_method_add_block(event)
        call = convert(event.args[0])
        block = event.args[1]
        opener = index_of_end(call) + 1
        scope = block_scope(block, opener, block.last)
        # A `.` call on a command's `do` block spans the block it is given.
        if call.type == :CALL && block_call?(event.args[0])
          call = node(:CALL, call.children, to_token(call, block.last))
        end
        node(:ITER, [call, scope], to_token(call, block.last))
      end

      # Whether an event is a command_call on a command given a `do` block.
      def block_call?(event)
        return false unless event.name == :command_call

        receiver = event.args[0]
        receiver.is_a?(Ev) && receiver.terminal && receiver.name == :method_add_block && receiver.args[1].name == :do_block
      end

      # The SCOPE of a block from its `{` or `do` to its `}` or `end`.
      def block_scope(block, opener, closer)
        block_var, body_event = block.args
        within(true) do |frame|
          args = nil
          if block_var
            close = block_var.last
            range = parameter_tokens(opener + 2, close - 1)
            locals = block_var.args[1] || []
            frame.it = :named
            args = parameters(block_var.args[0], range, locals) if range
            locals.each { |token| frame.names << token.text.to_sym } unless range
          end
          body = block_body(body_event)
          args ||= implicit_parameters(frame, closer)
          node(:SCOPE, [frame.names, args, body], span_of_tokens(opener, closer))
        end
      end

      def block_body(body_event)
        return body_statement(body_event) if body_event.is_a?(Ev) && body_event.name == :bodystmt

        statements(body_event)
      end

      # The ARGS of a block that reads `it` or numbered parameters, which
      # come first among its locals.
      def implicit_parameters(frame, closer)
        count = if frame.numbered.positive?
          names = (1..frame.numbered).map { |number| :"_#{number}" }
          frame.names.replace(names + (frame.names - names))
          frame.numbered
        elsif frame.it == true
          frame.names.unshift(:"<it>")
          1
        end
        return nil unless count

        line, column = end_of(closer)
        node(:ARGS, [count, nil, nil, nil, 0, nil, nil, nil, nil, nil], [line, column, line, column])
      end

      def on_lambda(event)
        params_event, body_event = event.args
        # The parameters follow the `->` directly, and an empty list was
        # built when the `->` was the last token taken.
        range = token_range(params_event)
        arrow = if params_event.is_a?(Ev) && params_event.name == :paren
                  matching_open(params_event.last, "(", ")") - 1
                elsif range
                  range.first - 1
                else
                  params_event.last
                end
        within(true) do |frame|
          frame.it = :named
          if params_event.is_a?(Ev) && params_event.name == :paren
            open = arrow + 1
            close = matching_open_forward(open)
            range = parameter_tokens(open + 1, close - 1)
            locals = lambda_locals(open + 1, close - 1)
            args = range ? parameters(params_event.args[0], range, locals) : node(:ARGS, no_parameters(start_of(open)).children, span_of_tokens(open, open))
            scope_start = start_of(open)
          else
            range = token_range(params_event)
            args = range ? parameters(params_event, range) : no_parameters(end_of(arrow))
            scope_start = range ? start_of(range[0]) : end_of(arrow)
          end
          body = block_body(body_event)
          scope = node(:SCOPE, [frame.names, args, body], [*scope_start, *end_of(event.last)])
          node(:LAMBDA, [scope], span_of_tokens(arrow, event.last))
        end
      end

      # The index of the `)` that closes the `(` at index `open`.
      def matching_open_forward(open)
        depth = 0
        (open...@tokens.size).each do |at|
          text = @tokens[at].text
          depth += 1 if text == "("
          depth -= 1 if text == ")"
          return at if depth.zero?
        end
      end

      # The block locals a lambda names after a `;` in its parameters.
      def lambda_locals(first, last)
        separator = (first..last).find { |at| @tokens[at].text == ";" }
        return [] unless separator

        @tokens[(separator + 1)..last].select { |token| token.type == :ident }
      end

      def on_yield(event)
        arguments = event.args[0]
        if arguments.is_a?(Ev) && arguments.name == :paren
          list = arguments_node(arguments.args[0])
          close = arguments.last
          opener = matching_open(close, "(", ")") - 1
          return node(:YIELD, [list], span_of_tokens(opener, close))
        end
        list = arguments_node(arguments)
        node(:YIELD, [list], from_token(index_of_start(list) - 1, list))
      end

      def on_yield0(event) = node(:YIELD, [nil], span_of_tokens(event.last, event.last))

      def on_zsuper(event) = node(:ZSUPER, [], span_of_tokens(event.last, event.last))

      def on_super(event)
        arguments = event.args[0]
        if arguments.is_a?(Ev) && arguments.name == :arg_paren
          list = arguments_node(arguments.args[0])
          close = arguments.last
          opener = matching_open(close, "(", ")") - 1
          return node(:SUPER, [list], span_of_tokens(opener, close))
        end
        list = arguments_node(arguments)
        # An argument such as a bare `super` names no token of its own, so
        # the search for the keyword starts from the last token read.
        keyword = opener_before(token_range(arguments)&.first || event.last, ["super"])
        node(:SUPER, [list], from_token(keyword, list))
      end

      def on_alias(event)
        old_name, new_name = event.args.map { |name| convert(name) }
        keyword = opener_before(index_of_start(old_name), ["alias"])
        node(:ALIAS, [old_name, new_name], from_token(keyword, new_name))
      end

      def on_var_alias(event)
        new_name, old_name = event.args
        keyword = opener_before(new_name.index, ["alias"])
        node(:VALIAS, [new_name.text.to_sym, old_name.text.to_sym], span_of_tokens(keyword, old_name.index))
      end

      def on_undef(event)
        names = event.args[0].map { |name| convert(name) }
        keyword = opener_before(index_of_start(names.first), ["undef"])
        node(:UNDEF, [names], from_token(keyword, names.last))
      end

      def on_defined(event)
        value = convert(event.args[0])
        keyword = opener_before(index_of_start(value), ["defined?"])
        parenthesized = @tokens[keyword + 1].text == "("
        last = parenthesized ? matching_open_forward(keyword + 1) : index_of_end(value)
        node(:DEFINED, [value], span_of_tokens(keyword, last))
      end

      # Pattern matching.

      # `case value; in pattern ...; end`, or `value in pattern` and
      # `value => pattern`, which match one pattern.
      def pattern_case(event)
        subject = convert(event.args[0])
        clause = event.args[1]
        keyword = index_of_start(subject) - 1
        if keyword >= 0 && @tokens[keyword].type == :k_case
          return node(:CASE3, [subject, in_clause(clause)], span_of_tokens(keyword, event.last))
        end

        operator = index_of_end(subject) + 1
        span = span_of_tokens(operator + 1, event.last)
        matched = top_pattern(clause.args[0], operator + 1, event.last)
        tested = @tokens[operator].type == :k_in ? [node(:TRUE, [], span), node(:FALSE, [], span)] : [nil, nil]
        node(:CASE3, [subject, node(:IN, [matched, *tested], span)], span_of_tokens(index_of_start(subject), event.last))
      end

      # The tokens a pattern can not reach past at its own nesting depth.
      PATTERN_ENDS = %i[k_then ; nl k_if_mod k_unless_mod].freeze

      def in_clause(clause)
        pattern_event, body_event, rest_event = clause.args
        keyword = literal_root(body_event).last
        keyword -= 1 until @tokens[keyword].type == :k_in
        last = keyword + 1
        depth = 0
        while last < @tokens.size
          depth += nesting(@tokens[last])
          break if depth.zero? && PATTERN_ENDS.include?(@tokens[last].type) && !(@tokens[last].type == :k_if_mod && false)

          last += 1
        end
        guard_end = last
        if %i[k_if_mod k_unless_mod].include?(@tokens[last].type)
          guard_end += 1 until guard_end + 1 >= @tokens.size || %i[k_then ; nl].include?(@tokens[guard_end + 1].type)
        else
          guard_end = last - 1
        end
        matched = top_pattern(pattern_event, keyword + 1, guard_end)
        body = statements(body_event)
        rest = if rest_event.nil? then nil
               elsif rest_event.name == :in then in_clause(rest_event)
               else statements(rest_event.args[0])
               end
        finish = if rest.nil? then through_semicolons(body)
                 elsif rest_event.name == :in then [rest.last_lineno, rest.last_column]
                 else through_semicolons(rest)
                 end
        node(:IN, [matched, body, rest], [*start_of(keyword), *finish])
      end

      # A pattern written from token `first` to token `last`, with the guard
      # that may follow it.
      def top_pattern(event, first, last)
        if event.is_a?(Ev) && %i[if_mod unless_mod].include?(event.name)
          guard = convert(event.args[0])
          keyword = index_of_start(guard) - 1
          matched = top_pattern(event.args[1], first, keyword - 1)
          span = [*start_of(first), guard.last_lineno, guard.last_column]
          return node(event.name == :if_mod ? :IF : :UNLESS, [cond(guard, span), matched, nil], span)
        end
        return pattern(event) unless event.is_a?(Ev) && %i[aryptn fndptn hshptn].include?(event.name)

        opener = { aryptn: "[", fndptn: "[", hshptn: "{" }.fetch(event.name)
        written_in_brackets = event.args[0].nil? && @tokens[first].text == opener && matching_close(first) == last
        return pattern(event) if written_in_brackets || event.args[0]

        case event.name
        when :aryptn then array_pattern(event, [first, last], top: true)
        when :fndptn then find_pattern(event, [first, last])
        else hash_pattern(event, [first, last])
        end
      end

      # The index of the bracket that closes the one at index `open`.
      def matching_close(open)
        depth = 0
        (open...@tokens.size).each do |at|
          depth += nesting(@tokens[at])
          return at if depth.zero?
        end
        nil
      end

      def pattern(event)
        return convert(event) unless event.is_a?(Ev)

        case event.name
        when :var_field then pattern_variable(event.args[0])
        when :binary then pattern_binary(event)
        when :aryptn then array_pattern(event)
        when :fndptn then find_pattern(event)
        when :hshptn then hash_pattern(event)
        when :var_ref then pinned(event)
        when :begin
          close = event.last
          node(:BLOCK, [convert(event.args[0])], span_of_tokens(matching_open(close, "(", ")") - 1, close))
        else convert(event)
        end
      end

      # A name a pattern binds the matched value to.
      def pattern_variable(token, span = nil)
        name = token.text.to_sym
        declare(name)
        node(local_write_type, [name, nil], span || span_of_tokens(token.index, token.index))
      end

      def pattern_binary(event)
        left = pattern(event.args[0])
        operator = event.args[1]
        if operator == :|
          right = pattern(event.args[2])
          return node(:OR, [left, right], [*outer_span(left).first(2), *outer_span(right).last(2)])
        end
        return convert(event) unless operator == :"=>"

        variable = pattern_variable(event.args[2].args[0])
        span = [*outer_span(left).first(2), *read_span(variable).last(2)]
        node(:HASH, [node(:LIST, [left, variable, nil], span)], span)
      end

      # A value read with `^`, which must be a local when it is a name.
      def pinned(event)
        token = event.args[0]
        return convert(event) unless token.index.positive? && @tokens[token.index - 1].text == "^"

        span = span_of_tokens(token.index - 1, token.index)
        value = convert(event)
        if token.type == :ident
          raise SyntaxError, "#{token.text}: no such local variable" unless local_name?(token.text.to_sym)

          return local_read(token.text.to_sym, span)
        end
        node(value.type, value.children, span)
      end

      # The brackets a pattern is written in, and the tokens between them:
      # [open, close, first, last], with first and last nil when empty.
      def pattern_brackets(event, constant)
        close = event.last + 1
        close += 1 while @tokens[close].type == :nl
        open = constant ? index_of_end(constant) + 1 : matching_open(close, @tokens[close].text == "}" ? "{" : "[", @tokens[close].text)
        open = matching_open(close, "(", ")") if @tokens[close].text == ")"
        first = open + 1
        first += 1 while @tokens[first].type == :nl
        last = close - 1
        last -= 1 while @tokens[last].type == :nl
        first > last ? [open, close, nil, nil] : [open, close, first, last]
      end

      # Where a pattern is written, with the brackets around it, which an
      # alternative or a binding around it spans.
      def outer_span(matched) = @pattern_spans[matched] || read_span(matched)

      def bracketed(built, constant, open, close)
        @pattern_spans[built] = span_of_tokens(constant ? index_of_start(constant) : open, close)
        built
      end

      # The span of a pattern written in brackets: what is between them,
      # from the constant when there is one, or the brackets when empty.
      def bracketed_span(constant, open, close, first, last)
        return span_of_tokens(constant ? index_of_start(constant) : open, close) if first.nil?
        return [constant.first_lineno, constant.first_column, *end_of(last)] if constant

        span_of_tokens(first, last)
      end

      def pattern_list(events)
        return nil if events.nil? || events.empty?

        items = events.map { |item| pattern(item) }
        node(:LIST, items + [nil], [*outer_span(items.first).first(2), *outer_span(items.last).last(2)])
      end

      # A rest: the name it binds, spanning its star, or the mark of an
      # unnamed one.
      def pattern_rest(field)
        return :NODE_SPECIAL_NO_NAME_REST if field.args[0].nil?

        token = field.args[0]
        pattern_variable(token, span_of_tokens(token.index - 1, token.index))
      end

      def array_pattern(event, bounds = nil, top: false)
        constant_event, pre, rest, post = event.args
        constant = constant_event && convert(constant_event)
        if bounds
          first, last = bounds
          span = span_of_tokens(first, last)
        else
          open, close, first, last = pattern_brackets(event, constant)
          span = bracketed_span(constant, open, close, first, last)
        end
        pre_list = pattern_list(pre)
        # A written first pattern in a list without brackets starts a LIST
        # that spans the whole list.
        pre_list = node(:LIST, pre_list.children, span) if top && pre_list
        rest_node = if rest then pattern_rest(rest)
                    elsif pre_list && trailing_comma?(pre.last, last) then :NODE_SPECIAL_NO_NAME_REST
                    end
        built = node(:ARYPTN, [constant, pre_list, rest_node, pattern_list(post)], span)
        bounds ? built : bracketed(built, constant, open, close)
      end

      def trailing_comma?(item, last)
        converted_end = token_range(item)&.last || item.last
        index = converted_end + 1
        index <= last && @tokens[index].text == ","
      end

      def find_pattern(event, bounds = nil)
        constant_event, before, middle, after = event.args
        constant = constant_event && convert(constant_event)
        if bounds
          span = span_of_tokens(*bounds)
        else
          open, close, first, last = pattern_brackets(event, constant)
          span = bracketed_span(constant, open, close, first, last)
        end
        built = node(:FNDPTN, [constant, pattern_rest(before), pattern_list(middle), pattern_rest(after)], span)
        bounds ? built : bracketed(built, constant, open, close)
      end

      def hash_pattern(event, bounds = nil)
        constant_event, pairs, rest = event.args
        constant = constant_event && convert(constant_event)
        if bounds
          first, last = bounds
        else
          open, close, first, last = pattern_brackets(event, constant)
        end
        span = bounds ? span_of_tokens(first, last) : bracketed_span(constant, open, close, first, last)
        return bracketed(node(:HSHPTN, [constant, nil, nil], span), constant, open, close) if first.nil?

        keywords = span_of_tokens(first, last)
        items = (pairs || []).flat_map do |label, value|
          key = label_symbol(label)
          [key, value.nil? ? pattern_variable_named(label) : pattern(value)]
        end
        list = items.empty? ? nil : node(:LIST, items + [nil], between(items.first, items.last))
        rest_node = if rest.nil? then nil
                    elsif rest.args[0] == :nil then :NODE_SPECIAL_NO_REST_KEYWORD
                    else pattern_variable(rest.args[0], keywords)
                    end
        hash = list || rest_node ? node(:HASH, [list], keywords) : nil
        built = node(:HSHPTN, [constant, hash, rest_node], span)
        bounds ? built : bracketed(built, constant, open, close)
      end

      # The name a `key:` pattern binds, spanning the label.
      def pattern_variable_named(label)
        name = label.text.chomp(":").to_sym
        declare(name)
        node(local_write_type, [name, nil], span_of_tokens(label.index, label.index))
      end

      # `BEGIN { ... }` runs before the program, so its body goes to the
      # front, and where it was written a BEGIN of nothing stays.
      def on_BEGIN(event)
        close = event.last
        span = span_of_tokens(matching_open(close, "{", "}"), close)
        @preexe << node(:BEGIN, [statements(event.args[0])], span)
        node(:BEGIN, [nil], span)
      end

      def on_END(event)
        close = event.last
        span = span_of_tokens(matching_open(close, "{", "}") - 1, close)
        node(:POSTEXE, [node(:SCOPE, [[], nil, statements(event.args[0])], span)], span)
      end

      # Strings. A literal's parts are joined the way MRI's parser joins
      # them: text stays a STR, one interpolation alone is an EVSTR, and
      # anything else is a DSTR of its leading text and a list of the parts
      # after it. Each list cell keeps the span its part had when added.

      # Text in a literal. `line_start` marks text that begins a line of a
      # squiggly heredoc, which is dedented.
      Text = Struct.new(:text, :span, :line_start)

      # An interpolation, `#{...}` or `#@name`.
      Interpolation = Struct.new(:body, :span)

      # Leading text and the cells after it, each [part, span].
      Joined = Struct.new(:text, :cells, :span, :line_start)

      def span_end(span, last) = [span[0], span[1], last[2], last[3]]

      def list_append(joined, part)
        joined.cells << [part, part.span]
        joined.span = span_end(joined.span, part.span)
        joined
      end

      def new_dstr(interpolation, span) = list_append(Joined.new(+"", [], span, false), interpolation)

      def str2dstr(text) = Joined.new(text.text, [], text.span, text.line_start)

      # The text a STR joined onto a literal is added to: the literal's own
      # leading text while it has no cells, or its last cell when that is
      # text.
      def string_literal_head(head)
        return nil unless head.is_a?(Joined)
        return head if head.cells.empty?

        last = head.cells.last[0]
        last.is_a?(Text) ? last : nil
      end

      def literal_concat(head, tail, span)
        return tail if head.nil?
        return head if tail.nil?

        head = new_dstr(head, span) if head.is_a?(Interpolation)
        if @heredoc_width
          head = str2dstr(head) if head.is_a?(Text)
          return list_append(head, tail)
        end
        case tail
        when Text
          target = head.is_a?(Text) ? head : string_literal_head(head)
          target ? target.text << tail.text : list_append(head, tail)
        when Joined
          if head.is_a?(Text)
            tail.text = head.text + tail.text
            return tail
          end
          target = string_literal_head(head)
          if target
            target.text << tail.text
            head.cells.concat(tail.cells)
          else
            head.cells << [Text.new(tail.text, span), span]
            head.cells.concat(tail.cells)
            head.span = span_end(head.span, span)
          end
        when Interpolation
          head = str2dstr(head) if head.is_a?(Text)
          list_append(head, tail)
        end
        head
      end

      # The parts a literal's content events list, in order.
      def literal_pieces(content)
        pieces = []
        while content.is_a?(Ev) && content.name.end_with?("_add")
          pieces.unshift(content.args[1])
          content = content.args[0]
        end
        pieces
      end

      SIMPLE_ESCAPES = { "n" => "\n", "t" => "\t", "s" => " ", "r" => "\r", "0" => "\0", "a" => "\a", "b" => "\b", "e" => "\e", "f" => "\f", "v" => "\v" }.freeze

      # The text an escaped literal stands for. A double-quoted literal reads
      # every escape, a single-quoted one only its backslash and delimiters.
      def cooked(text, quote)
        return text if quote == :raw

        # Escapes may write bytes that are not UTF-8, so the text is built
        # as bytes and read as UTF-8 at the end, as MRI reads a literal.
        result = "".b
        scanner_index = 0
        while scanner_index < text.size
          char = text[scanner_index]
          if char != "\\" || scanner_index + 1 >= text.size
            result << char.b
            scanner_index += 1
            next
          end
          following = text[scanner_index + 1]
          if quote != :double
            result << "\\" unless following == "\\" || quote.include?(following) || (quote.include?(" ") && following.match?(/\s/))
            result << following.b
            scanner_index += 2
            next
          end
          scanner_index += 2
          case following
          when "\n" then nil
          when "u"
            if text[scanner_index] == "{"
              close = text.index("}", scanner_index)
              text[scanner_index + 1...close].split.each { |code| result << code.to_i(16).chr(Encoding::UTF_8).b }
              scanner_index = close + 1
            else
              result << text[scanner_index, 4].to_i(16).chr(Encoding::UTF_8).b
              scanner_index += 4
            end
          when "x"
            digits = text[scanner_index, 2][/\A\h+/].to_s
            result << digits.to_i(16).chr
            scanner_index += digits.size
          when /[0-7]/
            digits = text[scanner_index - 1, 3][/\A[0-7]+/]
            result << digits.to_i(8).chr
            scanner_index += digits.size - 1
          else
            result << SIMPLE_ESCAPES.fetch(following, following).b
          end
        end
        result.force_encoding(Encoding::UTF_8)
      end

      # How a literal opened by `opener` reads its escapes: :double, :raw,
      # or the delimiters a single-quoted literal lets be escaped.
      def quoting(opener)
        text = @tokens[opener].text
        return :raw if text.match?(/\A<<[-~]?'/)
        return :double if text.start_with?("<<", "\"", "`", ":\"", "%Q", "%W", "%I", "%x") || text.match?(/\A%[^a-zA-Z]/)

        delimiter = text[-1]
        closing = { "(" => ")", "[" => "]", "{" => "}", "<" => ">" }.fetch(delimiter, delimiter)
        words = text.start_with?("%w", "%i") ? " " : ""
        [delimiter, closing].uniq.join + words
      end

      # One piece of a literal as a part. `raw` keeps a regexp's escapes.
      def literal_part(piece, raw)
        if piece.is_a?(Token)
          text = raw ? piece.text.to_s.dup : cooked(piece.text.to_s, @quote || :double)
          span = @heredoc_pieces[piece] || span_of_tokens(piece.index, piece.index)
          return Text.new(text, span, @heredoc_width && piece.column.zero?)
        end
        if piece.name == :string_dvar
          variable = piece.args[0]
          body = convert(variable)
          return Interpolation.new(body, from_token(index_of_start(body) - 1, body))
        end
        close = piece.last
        open = close
        depth = 0
        loop do
          depth += 1 if @tokens[open].type == :string_dend
          depth -= 1 if @tokens[open].type == :string_dbeg
          break if depth.zero?

          open -= 1
        end
        width = @heredoc_width
        @heredoc_width = nil
        body = statements(piece.args[0])
        @heredoc_width = width
        # Text alone between the braces is joined as text.
        return Joined.new(body.children[0].dup, [], read_span(body), false) if body.type == :STR

        Interpolation.new(body, span_of_tokens(open, close))
      end

      # A literal's parts joined, or nil for an empty literal.
      def join_literal(pieces, raw: false, regexp: false, quote: :double)
        saved = @quote
        @quote = quote
        result = nil
        first = nil
        pieces.each do |piece|
          part = literal_part(piece, raw)
          first ||= part.span
          span = span_end(first, part.span)
          result = regexp ? regexp_concat(result, part, span) : literal_concat(result, part, span)
        end
        result
      ensure
        @quote = saved
      end

      # A string literal as a part, spanning the literal.
      def string_part(event)
        return string_concat_part(event) if event.name == :string_concat

        content = event.args[0]
        width = nil
        if content.is_a?(Ripper::Engine::Dispatched)
          width = content.width
          content = content.source
        end
        pieces = literal_pieces(content)
        opener = literal_root(content).last
        heredoc = @tokens[opener].text.start_with?("<<")
        span = heredoc ? span_of_tokens(opener, opener) : span_of_tokens(opener, event.last)
        heredoc_spans(pieces, opener) if heredoc && width.nil?
        part = with_heredoc_width(width) { join_literal(pieces, quote: quoting(opener)) }
        part = heredoc_dedent(part, width) if width
        part.span = span if part
        part
      end

      # The event a literal's content chain starts from, built when its
      # opener was taken.
      def literal_root(content)
        content = content.args[0] while content.is_a?(Ev) && content.name.end_with?("_add")
        content
      end

      def with_heredoc_width(width)
        saved = @heredoc_width
        @heredoc_width = width
        yield
      ensure
        @heredoc_width = saved
      end

      # MRI reads a heredoc a line at a time. Text that runs onto a later
      # line is placed where that line starts, and the text after the last
      # interpolation, read once the heredoc has ended, at its opener.
      def heredoc_spans(pieces, opener)
        last = pieces.last
        pieces.each do |piece|
          next unless piece.is_a?(Token)

          if piece.equal?(last) && pieces.size > 1
            line, column = end_of(opener)
            @heredoc_pieces[piece] = [line, column, line, column]
          else
            first_line, = start_of(piece.index)
            line, column = end_of(piece.index)
            @heredoc_pieces[piece] = [line, 0, line, column] if line > first_line
          end
        end
      end

      # A squiggly heredoc's text with `width` columns of indentation taken
      # off each line, and the text of neighboring lines joined.
      def heredoc_dedent(part, width)
        return part if part.nil?

        if part.is_a?(Text)
          Ripper.dedent_string(part.text, width) if part.line_start
          return part
        end
        return part unless part.is_a?(Joined)

        previous = part
        Ripper.dedent_string(part.text, width) if part.line_start
        kept = []
        part.cells.each do |cell|
          item = cell[0]
          if item.is_a?(Text) || item.is_a?(Joined)
            Ripper.dedent_string(item.text, width) if item.is_a?(Text) && item.line_start
            if previous
              previous.text << item.text
              next
            end
            previous = item
          else
            previous = nil
          end
          kept << cell
        end
        part.cells.replace(kept)
        return Text.new(part.text, part.span, false) if part.cells.empty?

        part
      end

      def string_concat_part(event)
        first = string_part(event.args[0])
        second = string_part(event.args[1])
        span = span_end(first ? first.span : second.span, second ? second.span : first.span)
        literal_concat(first, second, span)
      end

      # The node for a part.
      def part_node(part)
        case part
        when Text then node(:STR, [part.text], part.span)
        when Interpolation then node(:EVSTR, [part.body], part.span)
        when Joined then joined_node(part, :DSTR)
        end
      end

      def joined_node(joined, type)
        first, *rest = joined.cells
        list = rest.empty? ? nil : node(:LIST, rest.map { |item, _| part_node(item) } + [nil], rest.first[1])
        node(type, [joined.text, first && part_node(first[0]), list], joined.span)
      end

      # A symbol, or a method name written bare as `alias` and `undef` take
      # them.
      def on_symbol_literal(event)
        symbol = event.args[0]
        return node(:SYM, [symbol.text.to_sym], span_of_tokens(symbol.index, symbol.index)) if symbol.is_a?(Token)

        token = symbol.args[0]
        node(:SYM, [token.text.to_sym], span_of_tokens(token.index - 1, token.index))
      end

      def on_string_literal(event) = finished_string(string_part(event), literal_span(event))

      def on_string_concat(event) = finished_string(string_part(event), literal_span(event))

      # The span of a string literal, or of literals written side by side.
      def literal_span(event)
        return span_end(literal_span(event.args[0]), literal_span(event.args[1])) if event.name == :string_concat

        content = event.args[0]
        content = content.source if content.is_a?(Ripper::Engine::Dispatched)
        opener = literal_root(content).last
        @tokens[opener].text.start_with?("<<") ? span_of_tokens(opener, opener) : span_of_tokens(opener, event.last)
      end

      # A string as a value: an empty one is a STR, and one interpolation
      # alone is a DSTR around it.
      def finished_string(part, span)
        return node(:STR, [+""], span) if part.nil?

        part = new_dstr(part, part.span) if part.is_a?(Interpolation)
        part_node(part)
      end

      def on_xstring_literal(event)
        content = event.args[0]
        width = nil
        if content.is_a?(Ripper::Engine::Dispatched)
          width = content.width
          content = content.source
        end
        pieces = literal_pieces(content)
        opener = literal_root(content).last
        heredoc = @tokens[opener].text.start_with?("<<")
        span = heredoc ? span_of_tokens(opener, opener) : span_of_tokens(opener, event.last)
        heredoc_spans(pieces, opener) if heredoc && width.nil?
        part = with_heredoc_width(width) { join_literal(pieces, quote: quoting(opener)) }
        part = heredoc_dedent(part, width) if width
        case part
        when nil then node(:XSTR, [+""], span)
        when Text then node(:XSTR, [part.text], span)
        when Joined
          part.span = span
          joined_node(part, :DXSTR)
        else joined_node(Joined.new(nil, [[part, span]], span, false), :DXSTR)
        end
      end

      def on_dyna_symbol(event)
        pieces = literal_pieces(event.args[0])
        opener = literal_root(event.args[0]).last
        span = span_of_tokens(opener, event.last)
        part = join_literal(pieces, quote: quoting(opener))
        case part
        when nil then node(:SYM, [:""], span)
        when Text then node(:SYM, [part.text.to_sym], span)
        when Joined
          part.span = span
          joined_node(part, :DSYM)
        else joined_node(Joined.new(nil, [[part, span]], span, false), :DSYM)
        end
      end

      # A regexp's parts are added one by one, without joining text.
      def regexp_concat(head, tail, span)
        return tail if head.nil?

        head = case head
               when Text then str2dstr(head)
               when Joined then head
               else list_append(Joined.new(nil, [], span, false), head)
               end
        list_append(head, tail)
      end

      REGEXP_OPTIONS = { "i" => Regexp::IGNORECASE, "x" => Regexp::EXTENDED, "m" => Regexp::MULTILINE, "n" => Regexp::NOENCODING }.freeze

      def on_regexp_literal(event)
        content, ending = event.args
        pieces = literal_pieces(content)
        opener = literal_root(content).last
        span = span_of_tokens(opener, ending.index)
        flags = ending.text[1..].to_s
        options = flags.chars.sum { |flag| REGEXP_OPTIONS.fetch(flag, 0) }
        part = join_literal(pieces, raw: true, regexp: true)
        case part
        when nil, Text then node(:REGX, [regexp_value(part ? part.text : "", options)], span)
        else
          part = Joined.new(+"", [[part, span]], span, false) if part.is_a?(Interpolation)
          part.span = span
          regexp = joined_node(part, :DREGX)
          flags.include?("o") ? node(:ONCE, [regexp], span) : regexp
        end
      end

      def regexp_value(source, options)
        Regexp.new(source, options)
      rescue RegexpError => error
        raise SyntaxError, error.message
      end

      # A `%w`, `%i`, `%W` or `%I` list.
      WORD_LISTS = %i[qwords_new qwords_add qsymbols_new qsymbols_add words_new words_add symbols_new symbols_add].freeze

      def word_list?(content) = content.is_a?(Ev) && WORD_LISTS.include?(content.name)

      def word_list(event)
        content = event.args[0]
        symbols = content.name.start_with?("qsymbols", "symbols")
        opener = event.last
        opener -= 1 until %i[qwords_beg words_beg qsymbols_beg symbols_beg].include?(@tokens[opener].type)
        quote = quoting(opener)
        words = literal_pieces(content).map do |word|
          part = join_literal(word.is_a?(Token) ? [word] : literal_pieces(word), quote: quote)
          part = new_dstr(part, part.span) if part.is_a?(Interpolation)
          if symbols
            part.is_a?(Text) ? node(:SYM, [part.text.to_sym], part.span) : joined_node(part, :DSYM)
          else
            part_node(part)
          end
        end
        span = span_of_tokens(opener, event.last)
        return node(:ZLIST, [], span) if words.empty?

        node(:LIST, words + [nil], span)
      end
    end

    # A Ripper that keeps what the grammar finds wrong with a program it can
    # still read, such as a `break` outside of a loop.
    class Reader < Ripper
      attr_reader :problems

      def compile_error(message)
        (@problems ||= []) << message
      end

      def on_parse_error(message) = compile_error(message)

      # MRI writes the warnings its parser gives under the name of the
      # source, `(none)` for a string, unless `-W0` turned them off.
      def warn(fmt, *args)
        return if $VERBOSE.nil?

        $stderr.write("#{filename}:#{lineno}: warning: #{format(fmt, *args)}\n")
      end

      %i[on_assign_error on_alias_error on_class_name_error on_param_error].each do |event|
        define_method(event) do |message, value|
          compile_error(message)
          value
        end
      end
    end

    # What the nodes of one parse share: the program's lines and tokens,
    # when the parse was asked to keep them.
    # The builder that made them answers where the keywords of a node are.
    Tree = Struct.new(:script_lines, :tokens, :builder)

    def self.__parse__(source, keep_script_lines: false, error_tolerant: false, keep_tokens: false)
      source = source.to_str
      # MRI names the source of a parse with no file `(none)`.
      reader = Reader.new(source, "(none)")
      bridge = Ripper::Engine::Bridge.new(reader)
      scanner = Ripper::Engine::Scanner.new(bridge, source, 1)
      grammar = Ripper::Engine::Grammar.new(bridge, scanner)
      bridge.grammar = grammar
      tree, tokens = begin
        grammar.syntax_tree
      rescue Ripper::Engine::SyntaxFailure => failure
        # The errors found before the parse stopped come first, as MRI's
        # parser lists every error it met.
        earlier = __error_messages__(source, grammar.taken, grammar.reported_errors)
        earlier += "\n" unless earlier.empty? || earlier.end_with?("\n")
        first = grammar.reported_errors.first
        line = if first
                 first[5] ? first[5][0] : first[3] && grammar.taken[first[3]]&.line
               else
                 failure.token && __token_place__(source, failure.token)[0]
               end
        quoted = failure.token && __parse_error_message__(source, failure)
        raise __syntax_error__(earlier + failure.message, earlier + (quoted || failure.message), line)
      end
      if reader.problems
        error = grammar.reported_errors.first
        line = error[5] ? error[5][0] : error[3] && tokens[error[3]]&.line
        raise __syntax_error__(__error_messages__(source, tokens, grammar.reported_errors), nil, line)
      end
      builder = Builder.new(tokens, source)
      root = builder.program(tree)
      kept = Tree.new(keep_script_lines ? source.lines : nil, keep_tokens ? builder.all_tokens(source) : nil, builder)
      builder.finish(root, kept)
    end

    # Every error's message, each with the line it names quoted beneath it,
    # one after another.
    def self.__error_messages__(source, tokens, errors)
      errors.map { |error| __error_message__(source, tokens, error) }.reduce("") do |text, part|
        text.empty? || text.end_with?("\n") ? text + part : "#{text}\n#{part}"
      end
    end

    # An error's message, with the line it names quoted beneath it and a
    # caret under the tokens it names. MRI quotes the line only while its
    # lexer is still on it when the error is reported.
    def self.__error_message__(source, tokens, error)
      message, first, last, at, after, place = error
      if place
        line, from, to = place
        quoted = __error_line__(source.lines[line - 1].to_s, from, to, true)
        return quoted ? "#{message}\n#{quoted}" : message
      end
      return message if first.nil? || tokens[first].line != tokens[at].line

      start = tokens[first]
      finish = tokens[last]
      ends_on_line = finish.line == start.line
      end_column = ends_on_line ? finish.column + finish.text.bytesize : nil
      begin_column = after ? end_column : start.column
      quoted = __error_line__(source.lines[start.line - 1].to_s, begin_column, end_column, ends_on_line)
      quoted ? "#{message}\n#{quoted}" : message
    end

    # A SyntaxError carrying the line it names, which
    # RubyVM::InstructionSequence puts ahead of its message.
    def self.__syntax_error__(message, quoted_message, line)
      error = SyntaxError.new(quoted_message || message)
      error.instance_variable_set(:@line, line)
      error
    end

    # A parse error's message with the line of the token it met quoted
    # beneath it.
    def self.__parse_error_message__(source, failure)
      line, column, finish = __token_place__(source, failure.token)
      quoted = __error_line__(source.lines[line - 1].to_s, column, finish, true)
      quoted ? "#{failure.message}\n#{quoted}" : failure.message
    end

    # Where a token stands, as its line and its first and last column. MRI
    # places the end of the input after the text of the last line.
    def self.__token_place__(source, token)
      return [token.line, token.column, token.column + token.text.to_s.bytesize] unless token.type == :eof

      lines = source.lines
      column = lines.last.to_s.chomp.bytesize
      [[lines.size, 1].max, column, column]
    end

    # The quoted line and caret MRI's parser writes under an error message,
    # or nil for a line too short to quote.
    def self.__error_line__(line, begin_column, end_column, one_line)
      margin = 30
      text = line.chomp.b
      line_end = text.bytesize
      point = end_column && line_end > end_column ? end_column : line_end
      from = point > margin ? point - margin : 0
      to = line_end - point > margin ? point + margin : line_end
      length = to - from
      before = after = ""
      if length > 4
        if from.positive?
          from -= 1
          from -= 1 while from.positive? && (text.getbyte(from) & 0xC0) == 0x80
          before = "..." if from.positive?
        end
        if to < line_end
          to -= 1
          to -= 1 while to > point && (text.getbyte(to) & 0xC0) == 0x80
          after = "..." if to < line_end
        end
      end
      return nil if length <= 4 && one_line

      mark = begin_column.clamp(from, point)
      caret = +""
      at = from
      if at <= mark
        while at < mark
          caret << (text.getbyte(at) == 9 ? "\t" : " ")
          at += 1
        end
        caret << "^"
        at += 1
      end
      limit = [point, line_end].min
      caret << "~" * (limit - at) if limit > at
      code = text.byteslice(from, to - from).force_encoding(line.encoding)
      "#{before}#{code}#{after}\n#{before}#{caret}\n"
    end

    # The nodes that call a method, which a backtrace location names.
    CALL_TYPES = %i[CALL FCALL VCALL OPCALL QCALL SUPER ZSUPER YIELD ATTRASGN].freeze

    # The node a Proc, a Method or a backtrace location was written as,
    # found by parsing the file it was read from again.
    def self.__of__(body, keep_script_lines: false, keep_tokens: false)
      case body
      when Proc, Method, UnboundMethod then path, line = body.source_location
      when Thread::Backtrace::Location then path, line = body.path, body.lineno
      else raise TypeError, "wrong argument type #{body.class} (expected method)"
      end
      return nil if path.nil?
      raise ArgumentError, "cannot get AST for method defined in eval" unless File.file?(path)

      source = File.read(path)
      root = __parse__(source, keep_script_lines: keep_script_lines, keep_tokens: keep_tokens)
      nodes = []
      walk = lambda do |value|
        case value
        when Node
          nodes << value
          value.children.each { |child| walk.call(child) }
        when Array then value.each { |item| walk.call(item) }
        end
      end
      walk.call(root)
      case body
      when Proc
        # A block knows the column it opened at, which tells it from others
        # on its line.
        opened = __block_position__(body)
        column = opened && source.lines[line - 1].to_s[0, opened[1]].bytesize
        blocks = nodes.select do |node|
          (node.type == :LAMBDA && node.first_lineno == line) || (node.type == :ITER && node.children[1].first_lineno == line)
        end
        # A lambda opens at its `{` or `do`, after the `->` its node starts
        # at, so the nearest one starting before that column is it.
        exact = column && blocks.find { |node| node.type == :ITER && node.children[1].first_column == column }
        lambda = column && blocks.select { |node| node.type == :LAMBDA && node.first_column <= column }.max_by(&:first_column)
        exact || lambda || blocks.first
      when Method, UnboundMethod
        name = body.original_name
        nodes.find do |node|
          named = node.type == :DEFN ? node.children[0] : node.type == :DEFS && node.children[1]
          named == name && node.first_lineno == line
        end
      else
        # A location that knows its column names a call written there: the
        # innermost one, which is the call still running, or for the place
        # an exception was raised the outermost, which is the call that
        # raised. Without a column it names the innermost call on its line.
        # A location counts its column in characters, and a node in bytes.
        column = body.instance_variable_get(:@column)
        column &&= source.lines[line - 1].to_s[0, column].bytesize
        calls = nodes.select { |node| CALL_TYPES.include?(node.type) && node.first_lineno == line }
        written = column && calls.select { |node| node.first_column == column }
        chosen = body.instance_variable_get(:@raised) ? written&.first : written&.last
        chosen || calls.last
      end
    end
  end
end
