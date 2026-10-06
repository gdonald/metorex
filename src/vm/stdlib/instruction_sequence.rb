# RubyVM::InstructionSequence for a program metorex reads. Metorex runs the
# syntax tree rather than instructions, so an instruction sequence holds the
# source it was compiled from and the node that source parses to. Its
# listing names the nodes MRI's compiler would have read, not YARV
# instructions.
class RubyVM
  class InstructionSequence
    # The compile options MRI answers, kept as given.
    COMPILE_OPTIONS = {
      inline_const_cache: true, peephole_optimization: true, tailcall_optimization: false,
      specialized_instruction: true, operands_unification: true, instructions_unification: false,
      debug_frozen_string_literal: false, coverage_enabled: true, debug_level: 0, frozen_string_literal: nil
    }.freeze

    # What `to_binary` writes ahead of the dumped sequence.
    BINARY_HEADER = "METOREX-ISEQ\0"

    class << self
      def __compile__(source, file, path, line)
        source = source.read if source.respond_to?(:read)
        source = source.to_str
        node = begin
          RubyVM::AbstractSyntaxTree.parse(source)
        rescue SyntaxError => error
          raise SyntaxError, "#{file}:#{error_line(error, line)}: #{error.message}"
        end
        __build__(source: source, path: file, absolute_path: path, label: "<compiled>", base_label: "<compiled>",
                  first_lineno: line, type: :top, node: node)
      end

      # The line a parse error names, counted from the line the source
      # starts on.
      def error_line(error, line)
        (error.instance_variable_get(:@line) || 1) + line - 1
      end

      def __compile_file__(file)
        source = File.read(file)
        node = begin
          RubyVM::AbstractSyntaxTree.parse(source)
        rescue SyntaxError => error
          raise SyntaxError, "#{file}:#{error_line(error, 1)}: #{error.message}"
        end
        __build__(source: source, path: file, absolute_path: File.expand_path(file), label: "<main>",
                  base_label: "<main>", first_lineno: 1, type: :top, node: node)
      end

      # The sequence a Proc or a Method was compiled into, or nil for one
      # with no Ruby source.
      def __of__(body)
        unless body.is_a?(Proc) || body.is_a?(Method) || body.is_a?(UnboundMethod)
          raise TypeError, "wrong argument type #{body.class} (expected Proc or Method)"
        end
        path, = body.source_location
        return nil if path.nil? || !File.file?(path)

        node = RubyVM::AbstractSyntaxTree.of(body)
        return nil if node.nil?

        source = File.read(path)
        if body.is_a?(Proc)
          enclosing = enclosing_label(RubyVM::AbstractSyntaxTree.parse(source), node)
          block_sequence(source, path, File.expand_path(path), node, enclosing)
        else
          method_sequence(source, path, File.expand_path(path), node)
        end
      end

      # The sequence a debugger frame runs. A method's is named by the method
      # alone and starts at its `def`, a block's at the block, and the
      # program's at line 0.
      def __frame_sequence__(location)
        path = location.path
        label = location.label.to_s
        block = label.start_with?("block ")
        prefix, owner = block ? label.split(" in ", 2) : [nil, label]
        base = owner.start_with?("<") ? owner : owner.split(/[#.]/).last
        label = block ? "#{prefix} in #{base}" : base
        type = block ? :block : (base == "<main>" ? :top : :method)
        line = base == "<main>" && !block ? 0 : frame_start(path, location.lineno, block ? nil : base)
        __build__(source: nil, path: path, absolute_path: location.absolute_path, label: label, base_label: base,
                  first_lineno: line, type: type, node: nil)
      end

      # The line the method named `name`, or the innermost block, holding
      # line `line` of the file at `path` starts on.
      def frame_start(path, line, name)
        return line unless path && File.file?(path)

        holders = []
        collect = lambda do |node|
          return unless node.is_a?(RubyVM::AbstractSyntaxTree::Node)

          holds = node.first_lineno <= line && line <= node.last_lineno
          named = node.type == :DEFN ? node.children[0] : node.type == :DEFS && node.children[1]
          holders << node if holds && (name ? named.to_s == name : %i[ITER LAMBDA].include?(node.type))
          node.children.each { |child| (child.is_a?(Array) ? child : [child]).each { |item| collect.call(item) } }
        end
        collect.call(RubyVM::AbstractSyntaxTree.parse(File.read(path)))
        holders.empty? ? line : holders.last.first_lineno
      rescue SyntaxError
        line
      end

      def __build__(**fields)
        sequence = __allocate_instance__(self)
        fields.each { |name, value| sequence.instance_variable_set(:"@#{name}", value) }
        sequence
      end

      def method_sequence(source, path, absolute_path, node)
        name = node.type == :DEFN ? node.children[0] : node.children[1]
        __build__(source: source, path: path, absolute_path: absolute_path, label: name.to_s, base_label: name.to_s,
                  first_lineno: node.first_lineno, type: :method, node: node)
      end

      def block_sequence(source, path, absolute_path, node, enclosing)
        levels, base = enclosing
        label = levels > 1 ? "block (#{levels} levels) in #{base}" : "block in #{base}"
        __build__(source: source, path: path, absolute_path: absolute_path, label: label, base_label: base,
                  first_lineno: node.first_lineno, type: :block, node: node)
      end

      # How many blocks deep `target` sits, and the label of the method,
      # class or program that holds them.
      def enclosing_label(root, target)
        path = []
        found = search(root, target, path)
        return [1, "<main>"] unless found

        # The block itself is the first level.
        levels = 1
        path.reverse_each do |ancestor|
          case ancestor.type
          when :ITER, :LAMBDA then levels += 1
          when :DEFN then return [levels, ancestor.children[0].to_s]
          when :DEFS then return [levels, ancestor.children[1].to_s]
          when :CLASS then return [levels, "<class:#{constant_name(ancestor.children[0])}>"]
          when :MODULE then return [levels, "<module:#{constant_name(ancestor.children[0])}>"]
          when :SCLASS then return [levels, "singleton class"]
          end
        end
        [levels, "<main>"]
      end

      def search(node, target, path)
        return false unless node.is_a?(RubyVM::AbstractSyntaxTree::Node)
        return true if same_place?(node, target)

        path.push(node)
        node.children.each do |child|
          items = child.is_a?(Array) ? child : [child]
          return true if items.any? { |item| search(item, target, path) }
        end
        path.pop
        false
      end

      def constant_name(path) = path.children.last.to_s

      # Whether two nodes, from separate parses of one file, are the same.
      def same_place?(node, other)
        node.type == other.type && node.first_lineno == other.first_lineno && node.first_column == other.first_column &&
          node.last_lineno == other.last_lineno && node.last_column == other.last_column
      end

      private :__compile__, :__compile_file__, :__of__, :__frame_sequence__, :__build__, :error_line, :frame_start,
              :method_sequence, :block_sequence, :enclosing_label, :search, :constant_name, :same_place?
    end

    attr_reader :path, :absolute_path, :label, :base_label, :first_lineno

    def inspect = "<RubyVM::InstructionSequence:#{@label}@#{@path}:#{@first_lineno}>"

    # Runs a sequence compiled from a whole program, at the top level.
    def eval
      raise TypeError, "Not a toplevel InstructionSequence" unless @type == :top

      TOPLEVEL_BINDING.eval(@source, @path, @first_lineno)
    end

    # MRI keeps no script lines for a sequence compiled from a string.
    def script_lines = nil

    # The line each statement starts on, which MRI reports as a line event,
    # with the call and return of a method or block around them.
    def trace_points
      return [] if @node.nil?

      points = statement_lines(body_node).map { |line| [line, :line] }
      case @type
      when :method then [[@node.first_lineno, :call], *points, [@node.last_lineno, :return]]
      when :block then [[@node.first_lineno, :b_call], *points, [@node.last_lineno, :b_return]]
      else points
      end
    end

    # The sequences of the methods, classes and blocks written directly in
    # this one.
    def each_child
      raise LocalJumpError, "no block given" unless block_given?

      children_of(body_node).each { |child| yield child }
      self
    end

    # The sequence as MRI's SimpleDataFormat Array, with the instructions
    # standing for the nodes of the syntax tree.
    def to_a
      scope = scope_node
      locals = scope ? scope.children[0].compact : []
      [
        "YARVInstructionSequence/SimpleDataFormat", 4, 0, 1,
        { arg_size: argument_count, local_size: locals.size, stack_max: 1, node_id: @node&.node_id,
          code_location: code_location, node_ids: [], parser: :metorex },
        @label, @path, @absolute_path, @first_lineno, @type, locals, parameters, [], instructions
      ]
    end

    # A listing of the nodes the sequence holds, under MRI's heading.
    def disasm
      heading = "== disasm: #<ISeq:#{@label}@#{@path}:#{@first_lineno} (#{code_location[0]},#{code_location[1]})-(#{code_location[2]},#{code_location[3]})>\n"
      lines = instructions.select { |item| item.is_a?(Array) }.each_with_index.map do |(name, *operands), index|
        format("%04d %-38s\n", index, [name, *operands.map(&:inspect)].join(" ").rstrip)
      end
      heading + lines.join
    end
    alias disassemble disasm

    # The sequence written as bytes `load_from_binary` reads back.
    def to_binary(extra = nil)
      BINARY_HEADER + Marshal.dump([@source, @path, @absolute_path, @label, @base_label, @first_lineno, @type, extra])
    end

    private_class_method def self.__load_from_binary__(binary)
      raise ArgumentError, "unknown binary format" unless binary.start_with?(BINARY_HEADER)

      source, path, absolute_path, label, base_label, line, type, = Marshal.load(binary.byteslice(BINARY_HEADER.bytesize..))
      node = source && RubyVM::AbstractSyntaxTree.parse(source)
      __build__(source: source, path: path, absolute_path: absolute_path, label: label, base_label: base_label,
                first_lineno: line, type: type, node: node)
    end

    private_class_method def self.__load_from_binary_extra_data__(binary)
      raise ArgumentError, "unknown binary format" unless binary.start_with?(BINARY_HEADER)

      Marshal.load(binary.byteslice(BINARY_HEADER.bytesize..)).last
    end

    private

    # The SCOPE a method, block or program runs.
    def scope_node
      return nil if @node.nil?
      return @node if @node.type == :SCOPE
      return @node.children.last if %i[DEFN DEFS].include?(@node.type)
      return @node.children[1] if @node.type == :ITER

      @node.children[0] if @node.type == :LAMBDA
    end

    def body_node = scope_node&.children&.last

    def argument_count
      arguments = scope_node&.children&.at(1)
      return 0 unless arguments.is_a?(RubyVM::AbstractSyntaxTree::Node)

      arguments.children[0].to_i + arguments.children[4].to_i
    end

    # The parameters as MRI's Hash names them: the count of leading ones
    # and the rest, keyword and block parameters by name.
    def parameters
      arguments = scope_node&.children&.at(1)
      return {} unless arguments.is_a?(RubyVM::AbstractSyntaxTree::Node)

      lead, _, _, _, post, _, rest, keywords, _, block = arguments.children
      described = {}
      described[:lead_num] = lead if lead.positive?
      described[:post_num] = post if post.positive?
      described[:rest_start] = lead if rest.is_a?(Symbol) && rest != :NODE_SPECIAL_EXCESSIVE_COMMA
      described[:keyword] = keyword_names(keywords) if keywords
      described[:use_block] = true if block
      described
    end

    def keyword_names(chain)
      names = []
      while chain.is_a?(RubyVM::AbstractSyntaxTree::Node)
        names << chain.children[0].children[0]
        chain = chain.children[1]
      end
      names
    end

    def code_location
      return [@first_lineno, 0, @first_lineno, 0] if @node.nil?

      [@node.first_lineno, @node.first_column, @node.last_lineno, @node.last_column]
    end

    # The nodes in the order MRI's compiler reads them: the line of each
    # statement, then each node before the nodes it holds, as [type,
    # values...].
    def instructions
      listed = []
      statements(body_node).each do |statement|
        listed << statement.first_lineno << :RUBY_EVENT_LINE
        walk(statement) { |node| listed << [node.type.downcase, *node.children.grep_v(RubyVM::AbstractSyntaxTree::Node).grep_v(Array)] }
      end
      listed << [:leave]
    end

    # Visits a node and those it holds. A method, class or block is a
    # sequence of its own, so only a block's call is visited.
    def walk(node, &visit)
      return unless node.is_a?(RubyVM::AbstractSyntaxTree::Node)

      visit.call(node)
      return walk(node.children[0], &visit) if node.type == :ITER
      return if %i[DEFN DEFS CLASS MODULE SCLASS LAMBDA].include?(node.type)

      node.children.each do |child|
        (child.is_a?(Array) ? child : [child]).each { |item| walk(item, &visit) }
      end
    end

    def statements(body)
      return [] unless body.is_a?(RubyVM::AbstractSyntaxTree::Node)

      body.type == :BLOCK ? body.children : [body]
    end

    # The lines the statements of a body start on.
    def statement_lines(body) = statements(body).map(&:first_lineno).uniq

    def children_of(node)
      found = []
      collect = lambda do |value|
        case value
        when RubyVM::AbstractSyntaxTree::Node
          case value.type
          when :DEFN, :DEFS
            found << self.class.send(:method_sequence, @source, @path, @absolute_path, value)
          when :CLASS, :MODULE
            name = "<#{value.type == :CLASS ? "class" : "module"}:#{self.class.send(:constant_name, value.children[0])}>"
            found << self.class.send(:__build__, source: @source, path: @path, absolute_path: @absolute_path, label: name,
                                          base_label: name, first_lineno: value.first_lineno, type: :class, node: value)
          when :ITER, :LAMBDA
            found << self.class.send(:block_sequence, @source, @path, @absolute_path, value, [1, @label])
          else
            value.children.each { |child| collect.call(child) }
          end
        when Array then value.each { |item| collect.call(item) }
        end
      end
      collect.call(node)
      found
    end
  end
end
