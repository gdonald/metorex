pub(super) const SOURCE: &str = r##"
# What MRI says about its own virtual machine. Metorex runs the syntax tree
# rather than instructions, so the names and settings here are MRI's, kept
# so code that reads them runs.
class RubyVM
  OPTS = ["direct threaded code", "operands unification", "inline method cache"]

  INSTRUCTION_NAMES = %w[
    nop getlocal setlocal getblockparam setblockparam getblockparamproxy getspecial setspecial
    getinstancevariable setinstancevariable getclassvariable setclassvariable opt_getconstant_path getconstant setconstant getglobal
    setglobal putnil putself putobject putspecialobject putstring putchilledstring concatstrings
    anytostring toregexp intern newarray pushtoarraykwsplat duparray duphash expandarray
    concatarray concattoarray pushtoarray splatarray splatkw newhash newrange pop
    dup dupn swap opt_reverse topn setn adjuststack defined
    definedivar checkmatch checkkeyword checktype defineclass definemethod definesmethod send
    sendforward opt_send_without_block opt_new objtostring opt_ary_freeze opt_hash_freeze opt_str_freeze opt_nil_p
    opt_str_uminus opt_duparray_send opt_newarray_send invokesuper invokesuperforward invokeblock leave throw
    jump branchif branchunless branchnil once opt_case_dispatch opt_plus opt_minus
    opt_mult opt_div opt_mod opt_eq opt_neq opt_lt opt_le opt_gt
    opt_ge opt_ltlt opt_and opt_or opt_aref opt_aset opt_length opt_size
    opt_empty_p opt_succ opt_not opt_regexpmatch2 invokebuiltin opt_invokebuiltin_delegate opt_invokebuiltin_delegate_leave getlocal_WC_0
    getlocal_WC_1 setlocal_WC_0 setlocal_WC_1 putobject_INT2FIX_0_ putobject_INT2FIX_1_ trace_nop trace_getlocal trace_setlocal
    trace_getblockparam trace_setblockparam trace_getblockparamproxy trace_getspecial trace_setspecial trace_getinstancevariable trace_setinstancevariable trace_getclassvariable
    trace_setclassvariable trace_opt_getconstant_path trace_getconstant trace_setconstant trace_getglobal trace_setglobal trace_putnil trace_putself
    trace_putobject trace_putspecialobject trace_putstring trace_putchilledstring trace_concatstrings trace_anytostring trace_toregexp trace_intern
    trace_newarray trace_pushtoarraykwsplat trace_duparray trace_duphash trace_expandarray trace_concatarray trace_concattoarray trace_pushtoarray
    trace_splatarray trace_splatkw trace_newhash trace_newrange trace_pop trace_dup trace_dupn trace_swap
    trace_opt_reverse trace_topn trace_setn trace_adjuststack trace_defined trace_definedivar trace_checkmatch trace_checkkeyword
    trace_checktype trace_defineclass trace_definemethod trace_definesmethod trace_send trace_sendforward trace_opt_send_without_block trace_opt_new
    trace_objtostring trace_opt_ary_freeze trace_opt_hash_freeze trace_opt_str_freeze trace_opt_nil_p trace_opt_str_uminus trace_opt_duparray_send trace_opt_newarray_send
    trace_invokesuper trace_invokesuperforward trace_invokeblock trace_leave trace_throw trace_jump trace_branchif trace_branchunless
    trace_branchnil trace_once trace_opt_case_dispatch trace_opt_plus trace_opt_minus trace_opt_mult trace_opt_div trace_opt_mod
    trace_opt_eq trace_opt_neq trace_opt_lt trace_opt_le trace_opt_gt trace_opt_ge trace_opt_ltlt trace_opt_and
    trace_opt_or trace_opt_aref trace_opt_aset trace_opt_length trace_opt_size trace_opt_empty_p trace_opt_succ trace_opt_not
    trace_opt_regexpmatch2 trace_invokebuiltin trace_opt_invokebuiltin_delegate trace_opt_invokebuiltin_delegate_leave trace_getlocal_WC_0 trace_getlocal_WC_1 trace_setlocal_WC_0 trace_setlocal_WC_1
    trace_putobject_INT2FIX_0_ trace_putobject_INT2FIX_1_ zjit_getinstancevariable zjit_setinstancevariable zjit_definedivar zjit_send zjit_opt_send_without_block zjit_objtostring
    zjit_opt_nil_p zjit_invokeblock zjit_opt_plus zjit_opt_minus zjit_opt_mult zjit_opt_div zjit_opt_mod zjit_opt_eq
    zjit_opt_neq zjit_opt_lt zjit_opt_le zjit_opt_gt zjit_opt_ge zjit_opt_ltlt zjit_opt_and zjit_opt_or
    zjit_opt_aref zjit_opt_aset zjit_opt_length zjit_opt_size zjit_opt_empty_p zjit_opt_succ zjit_opt_not zjit_opt_regexpmatch2
  ].map(&:freeze).freeze

  DEFAULT_PARAMS = {
    thread_vm_stack_size: 1048576,
    thread_machine_stack_size: 1048576,
    fiber_vm_stack_size: 131072,
    fiber_machine_stack_size: 524288
  }.freeze

  # The counters MRI keeps about its caches and object shapes. Metorex keeps
  # neither, so each one is zero.
  STAT_KEYS = %i[
    constant_cache_invalidations constant_cache_misses global_cvar_state
    next_shape_id shape_cache_size
  ].freeze

  __undefine_allocator__ self

  class << self
    undef_method :new

    def stat(wanted = nil)
      if wanted.is_a?(Symbol)
        raise ArgumentError, "unknown key: #{wanted}" unless STAT_KEYS.include?(wanted)
        return 0
      end
      unless wanted.nil? || wanted.is_a?(Hash)
        raise TypeError, "non-hash or symbol given"
      end
      filled = wanted.nil? ? {} : wanted
      STAT_KEYS.each { |key| filled[key] = 0 }
      filled
    end

    def keep_script_lines
      @__keep_script_lines ||= false
    end

    def keep_script_lines=(wanted)
      @__keep_script_lines = wanted
    end
  end

  # A program's syntax tree, as MRI's parser builds it.
  module AbstractSyntaxTree
    class Node
      attr_reader :type, :children, :first_lineno, :first_column, :last_lineno, :last_column

      def initialize(type, children, first_lineno, first_column, last_lineno, last_column)
        @type = type
        @children = children
        @first_lineno = first_lineno
        @first_column = first_column
        @last_lineno = last_lineno
        @last_column = last_column
      end

      def inspect
        "#<RubyVM::AbstractSyntaxTree::Node:#{@type}@#{@first_lineno}:#{@first_column}-#{@last_lineno}:#{@last_column}>"
      end

      attr_reader :node_id

      # Where the node is written, then where each keyword and operator MRI
      # records for its type is, nil for one not written.
      def locations
        recorded = @tree&.builder&.recorded_locations(self) || []
        [Location.new(@first_lineno, @first_column, @last_lineno, @last_column),
         *recorded.map { |span| span && Location.new(*span) }]
      end

      # The lines of the program, when it was parsed with
      # `keep_script_lines: true`.
      def script_lines = @tree&.script_lines

      # The text the node was written as.
      def source
        lines = script_lines
        return nil if lines.nil?
        return lines[@first_lineno - 1].byteslice(@first_column...@last_column) if @first_lineno == @last_lineno

        text = lines[@first_lineno - 1].byteslice(@first_column..)
        text += lines[@first_lineno...@last_lineno - 1].join
        text + lines[@last_lineno - 1].byteslice(0, @last_column)
      end

      # Every token of the program, when it was parsed with
      # `keep_tokens: true`.
      def all_tokens = @tree&.tokens

      # The tokens written within the node.
      def tokens
        all_tokens&.select do |_, _, _, (first_line, first_column, last_line, last_column)|
          ([first_line, first_column] <=> [@first_lineno, @first_column]) >= 0 &&
            ([last_line, last_column] <=> [@last_lineno, @last_column]) <= 0
        end
      end
    end

    # A span of a program's text.
    class Location
      attr_reader :first_lineno, :first_column, :last_lineno, :last_column

      def initialize(first_lineno, first_column, last_lineno, last_column)
        @first_lineno = first_lineno
        @first_column = first_column
        @last_lineno = last_lineno
        @last_column = last_column
      end

      def inspect
        "#<RubyVM::AbstractSyntaxTree::Location:@#{@first_lineno}:#{@first_column}-#{@last_lineno}:#{@last_column}>"
      end
    end

    def self.parse(source, **options)
      __load_abstract_syntax_tree__
      __parse__(source, **options)
    end

    def self.parse_file(path, **options)
      parse(File.read(path), **options)
    end

    def self.of(body, keep_script_lines: false, error_tolerant: false, keep_tokens: false)
      __load_abstract_syntax_tree__
      __of__(body, keep_script_lines: keep_script_lines, keep_tokens: keep_tokens)
    end

    def self.node_id_for_backtrace_location(location)
      __load_abstract_syntax_tree__
      __of__(location)&.node_id
    end
  end

  # A compiled program, method or block. The converter behind it is read the
  # first time one is made.
  class InstructionSequence
    class << self
      def compile(source, file = "<compiled>", path = nil, line = 1, options = nil, **)
        __load_instruction_sequence__
        __compile__(source, file, path || file, line)
      end
      alias new compile
      alias compile_prism compile
      alias compile_parsey compile

      def compile_file(file, options = nil, **)
        __load_instruction_sequence__
        __compile_file__(file)
      end
      alias compile_file_prism compile_file

      def of(body)
        __load_instruction_sequence__
        __of__(body)
      end

      def disasm(body) = of(body)&.disasm
      alias disassemble disasm

      def compile_option
        __load_instruction_sequence__
        @compile_option ||= COMPILE_OPTIONS.dup
      end

      def compile_option=(options)
        compile_option.merge!(options.to_hash) if options.respond_to?(:to_hash)
      end

      def load_from_binary(binary)
        __load_instruction_sequence__
        __load_from_binary__(binary)
      end

      def load_from_binary_extra_data(binary)
        __load_instruction_sequence__
        __load_from_binary_extra_data__(binary)
      end

      def __frame__(location)
        __load_instruction_sequence__
        __frame_sequence__(location)
      end
      private :__frame__
    end
  end
  __undefine_allocator__ InstructionSequence

  # MRI's just-in-time compilers. Metorex has none, so neither can be
  # turned on.
  module YJIT
    def self.enabled? = false
    def self.enable(**) = false
    def self.stats_enabled? = false
    def self.log_enabled? = false
    def self.trace_exit_locations_enabled? = false
    def self.runtime_stats(key = nil) = nil
    def self.stats_string = ""
    def self.reset_stats! = nil
  end

  module ZJIT
    def self.enabled? = false
    def self.enable = false
    def self.stats_enabled? = false
    def self.trace_exit_locations_enabled? = false
    def self.stats(key = nil) = nil
    def self.stats_string = ""
    def self.reset_stats! = nil
  end
end
"##;
