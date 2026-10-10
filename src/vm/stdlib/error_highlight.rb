# The snippet Ruby adds under the report of a NameError, TypeError or
# ArgumentError, with the part of the line that failed underlined. This is
# Ruby's error_highlight 0.7.2, which reads the node a backtrace names from
# RubyVM::AbstractSyntaxTree.
module ErrorHighlight
  VERSION = "0.7.2"
end

module ErrorHighlight
  class DefaultFormatter
    MIN_SNIPPET_WIDTH = 20

    def self.message_for(spot)
      # currently only a one-line code snippet is supported
      return "" unless spot[:first_lineno] == spot[:last_lineno]

      snippet      = spot[:snippet]
      first_column = spot[:first_column]
      last_column  = spot[:last_column]
      ellipsis     = "..."

      # truncate snippet to fit in the viewport
      if max_snippet_width && snippet.size > max_snippet_width
        available_width = max_snippet_width - ellipsis.size
        center          = first_column - max_snippet_width / 2

        visible_start  = last_column < available_width ? 0 : [center, 0].max
        visible_end    = visible_start + max_snippet_width
        visible_start  = snippet.size - max_snippet_width if visible_end > snippet.size

        prefix = visible_start.positive?    ? ellipsis : ""
        suffix = visible_end < snippet.size ? ellipsis : ""

        snippet = prefix + snippet[(visible_start + prefix.size)...(visible_end - suffix.size)] + suffix
        snippet << "\n" unless snippet.end_with?("\n")

        first_column -= visible_start
        last_column  = [last_column - visible_start, snippet.size - 1].min
      end

      indent = snippet[0...first_column].gsub(/[^\t]/, " ")
      marker = indent + "^" * (last_column - first_column)

      "\n\n#{ snippet }#{ marker }"
    end

    def self.max_snippet_width
      return if Ractor.current[:__error_highlight_max_snippet_width__] == :disabled

      Ractor.current[:__error_highlight_max_snippet_width__] ||= terminal_width
    end

    def self.max_snippet_width=(width)
      return Ractor.current[:__error_highlight_max_snippet_width__] = :disabled if width.nil?

      width = width.to_i

      if width < MIN_SNIPPET_WIDTH
        warn "'max_snippet_width' adjusted to minimum value of #{MIN_SNIPPET_WIDTH}."
        width = MIN_SNIPPET_WIDTH
      end

      Ractor.current[:__error_highlight_max_snippet_width__] = width
    end

    def self.terminal_width
      # lazy load io/console to avoid loading it when 'max_snippet_width' is manually set
      require "io/console"
      $stderr.winsize[1] if $stderr.tty?
    rescue LoadError, NoMethodError, SystemCallError
      # skip truncation when terminal window size is unavailable
    end
  end

  def self.formatter
    Ractor.current[:__error_highlight_formatter__] || DefaultFormatter
  end

  def self.formatter=(formatter)
    Ractor.current[:__error_highlight_formatter__] = formatter
  end
end


module ErrorHighlight
  # Identify the code fragment where a given exception occurred.
  #
  # Options:
  #
  # point_type: :name | :args
  #   :name (default) points to the method/variable name where the exception occurred.
  #   :args points to the arguments of the method call where the exception occurred.
  #
  # backtrace_location: Thread::Backtrace::Location
  #   It locates the code fragment of the given backtrace_location.
  #   By default, it uses the first frame of backtrace_locations of the given exception.
  #
  # Returns:
  #  {
  #    first_lineno: Integer,
  #    first_column: Integer,
  #    last_lineno: Integer,
  #    last_column: Integer,
  #    snippet: String,
  #    script_lines: [String],
  #  } | nil
  #
  # Limitations:
  #
  # Currently, ErrorHighlight.spot only supports a single-line code fragment.
  # Therefore, if the return value is not nil, first_lineno and last_lineno will have
  # the same value. If the relevant code fragment spans multiple lines
  # (e.g., Array#[] of <tt>ary[(newline)expr(newline)]</tt>), the method will return nil.
  # This restriction may be removed in the future.
  # ErrorHighlight.spot is defined below, over the nodes of RubyVM::AbstractSyntaxTree.

  class Spotter
    class NonAscii < Exception; end
    private_constant :NonAscii

    def initialize(node, point_type: :name, name: nil)
      @node = node
      @point_type = point_type
      @name = name

      # Not-implemented-yet options
      @arg = nil # Specify the index or keyword at which argument caused the TypeError/ArgumentError
      @multiline = false # Allow multiline spot

      @fetch = -> (lineno, last_lineno = lineno) do
        snippet = @node.script_lines[lineno - 1 .. last_lineno - 1].join("")
        snippet += "\n" unless snippet.end_with?("\n")

        # It requires some work to support Unicode (or multibyte) characters.
        # Tentatively, we stop highlighting if the code snippet has non-ascii characters.
        # See https://github.com/ruby/error_highlight/issues/4
        raise NonAscii unless snippet.ascii_only?

        snippet
      end
    end

    def spot
      return nil unless @node

      # In Ruby 3.2 or later, a nested constant access (like `Foo::Bar::Baz`)
      # is compiled to one instruction (opt_getconstant_path).
      # @node points to the node of the whole `Foo::Bar::Baz` even if `Foo`
      # or `Foo::Bar` causes NameError.
      # So we try to spot the sub-node that causes the NameError by using
      # `NameError#name`.
      case @node.type
      when :COLON2
        subnodes = []
        node = @node
        while node.type == :COLON2
          node2, const = node.children
          subnodes << node if const == @name
          node = node2
        end
        if node.type == :CONST || node.type == :COLON3
          if node.children.first == @name
            subnodes << node
          end

          # If we found only one sub-node whose name is equal to @name, use it
          return nil if subnodes.size != 1
          @node = subnodes.first
        else
          # Do nothing; opt_getconstant_path is used only when the const base is
          # NODE_CONST (`Foo`) or NODE_COLON3 (`::Foo`)
        end
      when :constant_path_node
        subnodes = []
        node = @node

        begin
          subnodes << node if node.name == @name
        end while (node = node.parent).is_a?(Prism::ConstantPathNode)

        if node.is_a?(Prism::ConstantReadNode) && node.name == @name
          subnodes << node
        end

        # If we found only one sub-node whose name is equal to @name, use it
        return nil if subnodes.size != 1
        @node = subnodes.first
      end

      case @node.type

      when :CALL, :QCALL
        case @point_type
        when :name
          spot_call_for_name
        when :args
          spot_call_for_args
        end

      when :ATTRASGN
        case @point_type
        when :name
          spot_attrasgn_for_name
        when :args
          spot_attrasgn_for_args
        end

      when :OPCALL
        case @point_type
        when :name
          spot_opcall_for_name
        when :args
          spot_opcall_for_args
        end

      when :FCALL
        case @point_type
        when :name
          spot_fcall_for_name
        when :args
          spot_fcall_for_args
        end

      when :VCALL
        spot_vcall

      when :OP_ASGN1
        case @point_type
        when :name
          spot_op_asgn1_for_name
        when :args
          spot_op_asgn1_for_args
        end

      when :OP_ASGN2
        case @point_type
        when :name
          spot_op_asgn2_for_name
        when :args
          spot_op_asgn2_for_args
        end

      when :CONST
        spot_vcall

      when :COLON2
        spot_colon2

      when :COLON3
        spot_vcall

      when :OP_CDECL
        spot_op_cdecl

      when :DEFN
        # There is nothing to highlight for the arguments of a method
        # definition, so just return nil instead of raising.
        return nil if @point_type != :name
        spot_defn

      when :DEFS
        return nil if @point_type != :name
        spot_defs

      when :LAMBDA
        return nil if @point_type != :name
        spot_lambda

      when :ITER
        return nil if @point_type != :name
        spot_iter

      when :call_node
        case @point_type
        when :name
          prism_spot_call_for_name
        when :args
          prism_spot_call_for_args
        end

      when :local_variable_operator_write_node
        case @point_type
        when :name
          prism_spot_local_variable_operator_write_for_name
        when :args
          prism_spot_local_variable_operator_write_for_args
        end

      when :call_operator_write_node
        case @point_type
        when :name
          prism_spot_call_operator_write_for_name
        when :args
          prism_spot_call_operator_write_for_args
        end

      when :index_operator_write_node
        case @point_type
        when :name
          prism_spot_index_operator_write_for_name
        when :args
          prism_spot_index_operator_write_for_args
        end

      when :constant_read_node
        prism_spot_constant_read

      when :constant_path_node
        prism_spot_constant_path

      when :constant_path_operator_write_node
        prism_spot_constant_path_operator_write

      when :def_node
        case @point_type
        when :name
          prism_spot_def_for_name
        when :args
          # There is nothing to highlight for the arguments of a method
          # definition (e.g. an ArgumentError for a missing required keyword
          # is spotted with point_type: :args). Return nil instead of raising
          # NotImplementedError, which would otherwise escape
          # Exception#detailed_message / #full_message.
          return nil
        end

      when :lambda_node
        case @point_type
        when :name
          prism_spot_lambda_for_name
        when :args
          # See the comment for :def_node above.
          return nil
        end

      when :block_node
        case @point_type
        when :name
          prism_spot_block_for_name
        when :args
          # See the comment for :def_node above.
          return nil
        end

      end

      if @snippet && @beg_column && @end_column && @beg_column < @end_column
        return {
          first_lineno: @beg_lineno,
          first_column: @beg_column,
          last_lineno: @end_lineno,
          last_column: @end_column,
          snippet: @snippet,
          script_lines: @node.script_lines,
        }
      else
        return nil
      end

    rescue NonAscii
      nil
    end

    private

    # Example:
    #   x.foo
    #    ^^^^
    #   x.foo(42)
    #    ^^^^
    #   x&.foo
    #    ^^^^^
    #   x[42]
    #    ^^^^
    #   x += 1
    #     ^
    def spot_call_for_name
      nd_recv, mid, nd_args = @node.children
      lineno = nd_recv.last_lineno
      lines = @fetch[lineno, @node.last_lineno]
      if mid == :[] && lines.match(/\G[\s)]*(\[(?:\s*\])?)/, nd_recv.last_column)
        @beg_column = $~.begin(1)
        @snippet = lines[/.*\n/]
        @beg_lineno = @end_lineno = lineno
        if nd_args
          if nd_recv.last_lineno == nd_args.last_lineno && @snippet.match(/\s*\]/, nd_args.last_column)
            @end_column = $~.end(0)
          end
        else
          if lines.match(/\G[\s)]*?\[\s*\]/, nd_recv.last_column)
            @end_column = $~.end(0)
          end
        end
      elsif lines.match(/\G[\s)]*?(\&?\.)(\s*?)(#{ Regexp.quote(mid) }).*\n/, nd_recv.last_column)
        lines = $` + $&
        @beg_column = $~.begin($2.include?("\n") ? 3 : 1)
        @end_column = $~.end(3)
        if i = lines[..@beg_column].rindex("\n")
          @beg_lineno = @end_lineno = lineno + lines[..@beg_column].count("\n")
          @snippet = lines[i + 1..]
          @beg_column -= i + 1
          @end_column -= i + 1
        else
          @snippet = lines
          @beg_lineno = @end_lineno = lineno
        end
      elsif mid.to_s =~ /\A\W+\z/ && lines.match(/\G\s*(#{ Regexp.quote(mid) })=.*\n/, nd_recv.last_column)
        @snippet = $` + $&
        @beg_lineno = @end_lineno = lineno
        @beg_column = $~.begin(1)
        @end_column = $~.end(1)
      end
    end

    # Example:
    #   x.foo(42)
    #         ^^
    #   x[42]
    #     ^^
    #   x += 1
    #        ^
    def spot_call_for_args
      _nd_recv, _mid, nd_args = @node.children
      if nd_args && nd_args.first_lineno == nd_args.last_lineno
        fetch_line(nd_args.first_lineno)
        @beg_column = nd_args.first_column
        @end_column = nd_args.last_column
      end
      # TODO: support @arg
    end

    # Example:
    #   x.foo = 1
    #    ^^^^^^
    #   x[42] = 1
    #    ^^^^^^
    def spot_attrasgn_for_name
      nd_recv, mid, nd_args = @node.children
      *nd_args, _nd_last_arg, _nil = nd_args.children
      fetch_line(nd_recv.last_lineno)
      if mid == :[]= && @snippet.match(/\G[\s)]*(\[)/, nd_recv.last_column)
        @beg_column = $~.begin(1)
        args_last_column = $~.end(0)
        if nd_args.last && nd_recv.last_lineno == nd_args.last.last_lineno
          args_last_column = nd_args.last.last_column
        end
        if @snippet.match(/[\s)]*\]\s*=/, args_last_column)
          @end_column = $~.end(0)
        end
      elsif @snippet.match(/\G[\s)]*(\.\s*#{ Regexp.quote(mid.to_s.sub(/=\z/, "")) }\s*=)/, nd_recv.last_column)
        @beg_column = $~.begin(1)
        @end_column = $~.end(1)
      end
    end

    # Example:
    #   x.foo = 1
    #           ^
    #   x[42] = 1
    #     ^^^^^^^
    #   x[] = 1
    #     ^^^^^
    def spot_attrasgn_for_args
      nd_recv, mid, nd_args = @node.children
      fetch_line(nd_recv.last_lineno)
      if mid == :[]= && @snippet.match(/\G[\s)]*\[/, nd_recv.last_column)
        @beg_column = $~.end(0)
        if nd_recv.last_lineno == nd_args.last_lineno
          @end_column = nd_args.last_column
        end
      elsif nd_args && nd_args.first_lineno == nd_args.last_lineno
        @beg_column = nd_args.first_column
        @end_column = nd_args.last_column
      end
      # TODO: support @arg
    end

    # Example:
    #   x + 1
    #     ^
    #   +x
    #   ^
    def spot_opcall_for_name
      nd_recv, op, nd_arg = @node.children
      fetch_line(nd_recv.last_lineno)
      if nd_arg
        # binary operator
        if @snippet.match(/\G[\s)]*(#{ Regexp.quote(op) })/, nd_recv.last_column)
          @beg_column = $~.begin(1)
          @end_column = $~.end(1)
        end
      else
        # unary operator
        if @snippet[...nd_recv.first_column].match(/(#{ Regexp.quote(op.to_s.sub(/@\z/, "")) })\s*\(?\s*\z/)
          @beg_column = $~.begin(1)
          @end_column = $~.end(1)
        end
      end
    end

    # Example:
    #   x + 1
    #       ^
    def spot_opcall_for_args
      _nd_recv, _op, nd_arg = @node.children
      if nd_arg && nd_arg.first_lineno == nd_arg.last_lineno
        # binary operator
        fetch_line(nd_arg.first_lineno)
        @beg_column = nd_arg.first_column
        @end_column = nd_arg.last_column
      end
    end

    # Example:
    #   foo(42)
    #   ^^^
    #   foo 42
    #   ^^^
    def spot_fcall_for_name
      mid, _nd_args = @node.children
      fetch_line(@node.first_lineno)
      if @snippet.match(/(#{ Regexp.quote(mid) })/, @node.first_column)
        @beg_column = $~.begin(1)
        @end_column = $~.end(1)
      end
    end

    # Example:
    #   foo(42)
    #       ^^
    #   foo 42
    #       ^^
    def spot_fcall_for_args
      _mid, nd_args = @node.children
      if nd_args && nd_args.first_lineno == nd_args.last_lineno
        fetch_line(nd_args.first_lineno)
        @beg_column = nd_args.first_column
        @end_column = nd_args.last_column
      end
    end

    # Example:
    #   foo
    #   ^^^
    def spot_vcall
      if @node.first_lineno == @node.last_lineno
        fetch_line(@node.last_lineno)
        @beg_column = @node.first_column
        @end_column = @node.last_column
      end
    end

    # Example:
    #   x[1] += 42
    #    ^^^    (for [])
    #   x[1] += 42
    #        ^  (for +)
    #   x[1] += 42
    #    ^^^^^^ (for []=)
    def spot_op_asgn1_for_name
      nd_recv, op, nd_args, _nd_rhs = @node.children
      fetch_line(nd_recv.last_lineno)
      if @snippet.match(/\G[\s)]*(\[)/, nd_recv.last_column)
        bracket_beg_column = $~.begin(1)
        args_last_column = $~.end(0)
        if nd_args && nd_recv.last_lineno == nd_args.last_lineno
          args_last_column = nd_args.last_column
        end
        if @snippet.match(/\s*\](\s*)(#{ Regexp.quote(op) })=()/, args_last_column)
          case @name
          when :[], :[]=
            @beg_column = bracket_beg_column
            @end_column = $~.begin(@name == :[] ? 1 : 3)
          when op
            @beg_column = $~.begin(2)
            @end_column = $~.end(2)
          end
        end
      end
    end

    # Example:
    #   x[1] += 42
    #     ^^^^^^^^
    def spot_op_asgn1_for_args
      nd_recv, mid, nd_args, nd_rhs = @node.children
      fetch_line(nd_recv.last_lineno)
      if mid == :[]= && @snippet.match(/\G\s*\[/, nd_recv.last_column)
        @beg_column = $~.end(0)
        if nd_recv.last_lineno == nd_rhs.last_lineno
          @end_column = nd_rhs.last_column
        end
      elsif nd_args && nd_args.first_lineno == nd_rhs.last_lineno
        @beg_column = nd_args.first_column
        @end_column = nd_rhs.last_column
      end
      # TODO: support @arg
    end

    # Example:
    #   x.foo += 42
    #    ^^^     (for foo)
    #   x.foo += 42
    #         ^  (for +)
    #   x.foo += 42
    #    ^^^^^^^ (for foo=)
    def spot_op_asgn2_for_name
      nd_recv, _qcall, attr, op, _nd_rhs = @node.children
      fetch_line(nd_recv.last_lineno)
      if @snippet.match(/\G[\s)]*(\.)\s*#{ Regexp.quote(attr) }()\s*(#{ Regexp.quote(op) })(=)/, nd_recv.last_column)
        case @name
        when attr
          @beg_column = $~.begin(1)
          @end_column = $~.begin(2)
        when op
          @beg_column = $~.begin(3)
          @end_column = $~.end(3)
        when :"#{ attr }="
          @beg_column = $~.begin(1)
          @end_column = $~.end(4)
        end
      end
    end

    # Example:
    #   x.foo += 42
    #            ^^
    def spot_op_asgn2_for_args
      _nd_recv, _qcall, _attr, _op, nd_rhs = @node.children
      if nd_rhs.first_lineno == nd_rhs.last_lineno
        fetch_line(nd_rhs.first_lineno)
        @beg_column = nd_rhs.first_column
        @end_column = nd_rhs.last_column
      end
    end

    # Example:
    #   Foo::Bar
    #      ^^^^^
    def spot_colon2
      nd_parent, const = @node.children
      if nd_parent.last_lineno == @node.last_lineno
        fetch_line(nd_parent.last_lineno)
        @beg_column = nd_parent.last_column
        @end_column = @node.last_column
      else
        fetch_line(@node.last_lineno)
        if @snippet[...@node.last_column].match(/#{ Regexp.quote(const) }\z/)
          @beg_lineno = @end_lineno = @node.last_lineno
          @beg_column = $~.begin(0)
          @end_column = $~.end(0)
        end
      end
    end

    # Example:
    #   Foo::Bar += 1
    #      ^^^^^^^^
    def spot_op_cdecl
      nd_lhs, op, _nd_rhs = @node.children
      *nd_parent_lhs, _const = nd_lhs.children
      if @name == op
        fetch_line(nd_lhs.last_lineno)
        if @snippet.match(/\G\s*(#{ Regexp.quote(op) })=/, nd_lhs.last_column)
          @beg_column = $~.begin(1)
          @end_column = $~.end(1)
        end
      else
        # constant access error
        @end_column = nd_lhs.last_column
        if nd_parent_lhs.empty? # example: ::C += 1
          if nd_lhs.first_lineno == nd_lhs.last_lineno
            fetch_line(nd_lhs.last_lineno)
            @beg_column = nd_lhs.first_column
          end
        else # example: Foo::Bar::C += 1
          if nd_parent_lhs.last.last_lineno == nd_lhs.last_lineno
            fetch_line(nd_lhs.last_lineno)
            @beg_column = nd_parent_lhs.last.last_column
          end
        end
      end
    end

    # Example:
    #   def bar; end
    #       ^^^
    def spot_defn
      mid, = @node.children
      fetch_line(@node.first_lineno)
      if @snippet.match(/\Gdef\s+(#{ Regexp.quote(mid) }\b)/, @node.first_column)
        @beg_column = $~.begin(1)
        @end_column = $~.end(1)
      end
    end

    # Example:
    #   def Foo.bar; end
    #          ^^^^
    def spot_defs
      nd_recv, mid, = @node.children
      fetch_line(nd_recv.last_lineno)
      if @snippet.match(/\G\s*(\.\s*#{ Regexp.quote(mid) }\b)/, nd_recv.last_column)
        @beg_column = $~.begin(1)
        @end_column = $~.end(1)
      end
    end

    # Example:
    #   -> { ... }
    #   ^^
    def spot_lambda
      fetch_line(@node.first_lineno)
      if @snippet.match(/\G->/, @node.first_column)
        @beg_column = $~.begin(0)
        @end_column = $~.end(0)
      end
    end

    # Example:
    #   lambda { ... }
    #          ^
    #   define_method :foo do
    #                      ^^
    def spot_iter
      _nd_fcall, nd_scope = @node.children
      fetch_line(nd_scope.first_lineno)
      if @snippet.match(/\G(?:do\b|\{)/, nd_scope.first_column)
        @beg_column = $~.begin(0)
        @end_column = $~.end(0)
      end
    end

    def fetch_line(lineno)
      @beg_lineno = @end_lineno = lineno
      @snippet = @fetch[lineno]
    end

    # Take a location from the prism parser and set the necessary instance
    # variables.
    def prism_location(location)
      @beg_lineno = location.start_line
      @beg_column = location.start_column
      @end_lineno = location.end_line
      @end_column = location.end_column
      @snippet = @fetch[@beg_lineno, @end_lineno]
    end

    # Example:
    #   x.foo
    #    ^^^^
    #   x.foo(42)
    #    ^^^^
    #   x&.foo
    #    ^^^^^
    #   x[42]
    #    ^^^^
    #   x.foo = 1
    #    ^^^^^^
    #   x[42] = 1
    #    ^^^^^^
    #   x + 1
    #     ^
    #   +x
    #   ^
    #   foo(42)
    #   ^^^
    #   foo 42
    #   ^^^
    #   foo
    #   ^^^
    def prism_spot_call_for_name
      # Explicitly turn off foo.() syntax because error_highlight expects this
      # to not work.
      return nil if @node.name == :call && @node.message_loc.nil?

      location = @node.message_loc || @node.call_operator_loc || @node.location
      location = @node.call_operator_loc.join(location) if @node.call_operator_loc&.start_line == location.start_line

      # If the method name ends with "=" but the message does not, then this is
      # a method call using the "attribute assignment" syntax
      # (e.g., foo.bar = 1). In this case we need to go retrieve the = sign and
      # add it to the location.
      if (name = @node.name).end_with?("=") && !@node.message.end_with?("=")
        location = location.adjoin("=")
      end

      prism_location(location)

      if !name.end_with?("=") && !name.match?(/[[:alpha:]_\[]/)
        # If the method name is an operator, then error_highlight only
        # highlights the first line.
        fetch_line(location.start_line)
      end
    end

    # Example:
    #   x.foo(42)
    #         ^^
    #   x[42]
    #     ^^
    #   x.foo = 1
    #           ^
    #   x[42] = 1
    #     ^^^^^^^
    #   x[] = 1
    #     ^^^^^
    #   x + 1
    #       ^
    #   foo(42)
    #       ^^
    #   foo 42
    #       ^^
    def prism_spot_call_for_args
      # Disallow highlighting arguments if there are no arguments.
      return if @node.arguments.nil?

      # Explicitly turn off foo.() syntax because error_highlight expects this
      # to not work.
      return nil if @node.name == :call && @node.message_loc.nil?

      if @node.name == :[]= && @node.opening == "[" && (@node.arguments&.arguments || []).length == 1
        prism_location(@node.opening_loc.copy(start_offset: @node.opening_loc.start_offset + 1).join(@node.arguments.location))
      else
        prism_location(@node.arguments.location)
      end
    end

    # Example:
    #   x += 1
    #     ^
    def prism_spot_local_variable_operator_write_for_name
      prism_location(@node.binary_operator_loc.chop)
    end

    # Example:
    #   x += 1
    #        ^
    def prism_spot_local_variable_operator_write_for_args
      prism_location(@node.value.location)
    end

    # Example:
    #   x.foo += 42
    #    ^^^     (for foo)
    #   x.foo += 42
    #         ^  (for +)
    #   x.foo += 42
    #    ^^^^^^^ (for foo=)
    def prism_spot_call_operator_write_for_name
      if !@name.start_with?(/[[:alpha:]_]/)
        prism_location(@node.binary_operator_loc.chop)
      else
        location = @node.message_loc
        if @node.call_operator_loc.start_line == location.start_line
          location = @node.call_operator_loc.join(location)
        end

        location = location.adjoin("=") if @name.end_with?("=")
        prism_location(location)
      end
    end

    # Example:
    #   x.foo += 42
    #            ^^
    def prism_spot_call_operator_write_for_args
      prism_location(@node.value.location)
    end

    # Example:
    #   x[1] += 42
    #    ^^^    (for [])
    #   x[1] += 42
    #        ^  (for +)
    #   x[1] += 42
    #    ^^^^^^ (for []=)
    def prism_spot_index_operator_write_for_name
      case @name
      when :[]
        prism_location(@node.opening_loc.join(@node.closing_loc))
      when :[]=
        prism_location(@node.opening_loc.join(@node.closing_loc).adjoin("="))
      else
        # Explicitly turn off foo[] += 1 syntax when the operator is not on
        # the same line because error_highlight expects this to not work.
        return nil if @node.binary_operator_loc.start_line != @node.opening_loc.start_line

        prism_location(@node.binary_operator_loc.chop)
      end
    end

    # Example:
    #   x[1] += 42
    #     ^^^^^^^^
    def prism_spot_index_operator_write_for_args
      opening_loc =
        if @node.arguments.nil?
          @node.opening_loc.copy(start_offset: @node.opening_loc.start_offset + 1)
        else
          @node.arguments.location
        end

      prism_location(opening_loc.join(@node.value.location))
    end

    # Example:
    #   Foo
    #   ^^^
    def prism_spot_constant_read
      prism_location(@node.location)
    end

    # Example:
    #   Foo::Bar
    #      ^^^^^
    def prism_spot_constant_path
      if @node.parent && @node.parent.location.end_line == @node.location.end_line
        fetch_line(@node.parent.location.end_line)
        prism_location(@node.delimiter_loc.join(@node.name_loc))
      else
        fetch_line(@node.location.end_line)
        location = @node.name_loc
        location = @node.delimiter_loc.join(location) if @node.delimiter_loc.end_line == location.start_line
        prism_location(location)
      end
    end

    # Example:
    #   Foo::Bar += 1
    #      ^^^^^^^^
    def prism_spot_constant_path_operator_write
      if @name == (target = @node.target).name
        prism_location(target.delimiter_loc.join(target.name_loc))
      else
        prism_location(@node.binary_operator_loc.chop)
      end
    end

    # Example:
    #   def foo()
    #       ^^^
    def prism_spot_def_for_name
      location = @node.name_loc
      location = @node.operator_loc.join(location) if @node.operator_loc
      prism_location(location)
    end

    # Example:
    #   -> x, y { }
    #   ^^
    def prism_spot_lambda_for_name
      prism_location(@node.operator_loc)
    end

    # Example:
    #   lambda { }
    #          ^
    #   define_method :foo do |x, y|
    #                      ^
    def prism_spot_block_for_name
      prism_location(@node.opening_loc)
    end
  end

  private_constant :Spotter
end


module ErrorHighlight
  module CoreExt
    private def generate_snippet
      if ArgumentError === self && message =~ /\A(?:wrong number of arguments|missing keyword[s]?|unknown keyword[s]?|no keywords accepted)\b/
        locs = self.backtrace_locations
        return "" if locs.size < 2
        callee_loc, caller_loc = locs
        callee_spot = ErrorHighlight.spot(self, backtrace_location: callee_loc, point_type: :name)
        caller_spot = ErrorHighlight.spot(self, backtrace_location: caller_loc, point_type: :name, name: callee_loc.label.to_s[/[\w?!=]+\z/])
        if caller_spot && callee_spot &&
            caller_loc.path == callee_loc.path &&
            caller_loc.lineno == callee_loc.lineno &&
            caller_spot == callee_spot
          callee_loc = callee_spot = nil
        end
        ret = +"\n"
        [["caller", caller_loc, caller_spot], ["callee", callee_loc, callee_spot]].each do |header, loc, spot|
          out = nil
          if loc
            out = "    #{ header }: #{ loc.path }:#{ loc.lineno }"
            if spot
              _, _, snippet, highlight = ErrorHighlight.formatter.message_for(spot).lines
              out += "\n    | #{ snippet }      #{ highlight }"
            else
              # do nothing
            end
          end
          ret << "\n" + out if out
        end
        ret
      else
        spot = ErrorHighlight.spot(self)
        return "" unless spot
        return ErrorHighlight.formatter.message_for(spot)
      end
    end

    if Exception.method_defined?(:detailed_message)
      def detailed_message(highlight: false, error_highlight: true, **)
        return super unless error_highlight
        snippet = generate_snippet
        if highlight
          snippet = snippet.gsub(/.+/) { "\e[1m" + $& + "\e[m" }
        end
        super + snippet
      end
    else
      # This is a marker to let `DidYouMean::Correctable#original_message` skip
      # the following method definition of `to_s`.
      # See https://github.com/ruby/did_you_mean/pull/152
      SKIP_TO_S_FOR_SUPER_LOOKUP = true
      private_constant :SKIP_TO_S_FOR_SUPER_LOOKUP

      def to_s
        msg = super
        snippet = generate_snippet
        if snippet != "" && !msg.include?(snippet)
          msg + snippet
        else
          msg
        end
      end
    end
  end

  NameError.prepend(CoreExt)

  if Exception.method_defined?(:detailed_message)
    # ErrorHighlight is enabled for TypeError and ArgumentError only when Exception#detailed_message is available.
    # This is because changing ArgumentError#message is highly incompatible.
    TypeError.prepend(CoreExt)
    ArgumentError.prepend(CoreExt)
  end
end

module ErrorHighlight
  # The node types a spot can be found on, by the child that names the
  # method or constant each one reaches.
  NAMED_NODES = {
    CALL: 1, QCALL: 1, OPCALL: 1, ATTRASGN: 1, OP_ASGN2: 2, OP_ASGN1: 1,
    FCALL: 0, VCALL: 0, CONST: 0, COLON3: 0, COLON2: 1,
  }.freeze
  private_constant :NAMED_NODES

  def self.spot(obj, **opts)
    case obj
    when Exception
      exc = obj
      loc = opts[:backtrace_location]
      point_type = opts.fetch(:point_type, :name)
      if loc
        # The place an error was raised names the call that raised it more
        # closely than its line does.
        hint = exc.__spot_hint__
        node =
          if hint && hint[0] == loc.path && hint[1] == loc.lineno && opts[:name].nil?
            node_at(*hint)
          else
            node_at_location(loc, opts[:name])
          end
        name = nil
      else
        point_type = :args if TypeError === exc || ArgumentError === exc
        hint = exc.__spot_hint__
        return nil unless hint

        path, lineno, column, named = hint
        name = exc.name if NameError === exc
        node = node_at(path, lineno, column, named)
      end
      return nil unless node

      Spotter.new(node, point_type: point_type, name: name).spot
    when RubyVM::AbstractSyntaxTree::Node
      Spotter.new(obj, **opts).spot
    else
      raise TypeError, "Exception is expected"
    end
  rescue SyntaxError,
         SystemCallError, # file not found or something
         ArgumentError # eval'ed code
    return nil
  end

  # The tree of the program file at `path`, with its lines kept. A program
  # given with `-e` or on standard input has no file, and no tree.
  def self.tree_of(path)
    return nil unless File.file?(path)

    source = File.read(path)

    [RubyVM::AbstractSyntaxTree.parse(source, keep_script_lines: true), source]
  end
  private_class_method :tree_of

  def self.each_node(node, &block)
    return unless node.is_a?(RubyVM::AbstractSyntaxTree::Node)

    yield node
    node.children.each { |child| each_node(child, &block) }
  end
  private_class_method :each_node

  # The node an error was raised in: one starting where it started that
  # reaches the method or constant it names.
  def self.node_at(path, lineno, column, name)
    root, source = tree_of(path)
    return nil unless root

    column = source.lines[lineno - 1].to_s[0, column].bytesize
    found = []
    each_node(root) do |node|
      next unless NAMED_NODES.key?(node.type)
      next unless node.first_lineno == lineno && node.first_column == column

      found << node
    end
    found.find { |node| name && node.children[NAMED_NODES[node.type]] == name } || found.first
  end
  private_class_method :node_at

  # The node a backtrace location names: the method or block defined there
  # when the location is inside it, and otherwise the call to `name` written
  # on its line.
  def self.node_at_location(loc, name)
    root, = tree_of(loc.path)
    return nil unless root

    defined = loc.label.to_s[/[\w?!=]+\z/]&.to_sym
    calls = []
    definitions = []
    each_node(root) do |node|
      next unless node.first_lineno == loc.lineno

      case node.type
      when :DEFN then definitions << node if node.children[0] == defined
      when :DEFS then definitions << node if node.children[1] == defined
      when :CALL, :QCALL, :FCALL, :VCALL, :OPCALL then calls << node
      end
    end
    return definitions.first if definitions.any? && !loc.label.to_s.start_with?("<")

    calls.find { |node| node.children[NAMED_NODES[node.type]].to_s == name.to_s } || calls.last
  end
  private_class_method :node_at_location
end
