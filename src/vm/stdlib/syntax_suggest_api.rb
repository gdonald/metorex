# The report Ruby adds to a SyntaxError, naming the lines around a missing
# or extra `end`. This is Ruby's syntax_suggest, gathered into one file.
# frozen_string_literal: true
# frozen_string_literal: true

module SyntaxSuggest
  VERSION = "2.0.3"
end

require "tmpdir"
require "stringio"
require "pathname"
require "timeout"

# We need Ripper loaded for `Prism.lex_compat` even if we're using Prism
# for lexing and parsing
require "ripper"

# Prism is the new parser, replacing Ripper
#
# We need to "dual boot" both for now because syntax_suggest
# supports older rubies that do not ship with syntax suggest.
#
# We also need the ability to control loading of this library
# so we can test that both modes work correctly in CI.
if (value = ENV["SYNTAX_SUGGEST_DISABLE_PRISM"])
  warn "Skipping loading prism due to SYNTAX_SUGGEST_DISABLE_PRISM=#{value}"
else
  begin
    require "prism"
  rescue LoadError
  end
end

module SyntaxSuggest
  # Used to indicate a default value that cannot
  # be confused with another input.
  DEFAULT_VALUE = Object.new.freeze

  class Error < StandardError; end
  TIMEOUT_DEFAULT = ENV.fetch("SYNTAX_SUGGEST_TIMEOUT", 1).to_i

  # SyntaxSuggest.use_prism_parser? [Private]
  #
  # Tells us if the prism parser is available for use
  # or if we should fallback to `Ripper`
  def self.use_prism_parser?
    defined?(Prism)
  end

  # SyntaxSuggest.handle_error [Public]
  #
  # Takes a `SyntaxError` exception, uses the
  # error message to locate the file. Then the file
  # will be analyzed to find the location of the syntax
  # error and emit that location to stderr.
  #
  # Example:
  #
  #   begin
  #     require 'bad_file'
  #   rescue => e
  #     SyntaxSuggest.handle_error(e)
  #   end
  #
  # By default it will re-raise the exception unless
  # `re_raise: false`. The message output location
  # can be configured using the `io: $stderr` input.
  #
  # If a valid filename cannot be determined, the original
  # exception will be re-raised (even with
  # `re_raise: false`).
  def self.handle_error(e, re_raise: true, io: $stderr)
    unless e.is_a?(SyntaxError)
      io.puts("SyntaxSuggest: Must pass a SyntaxError, got: #{e.class}")
      raise e
    end

    file = PathnameFromMessage.new(e.message, io: io).call.name
    raise e unless file

    io.sync = true

    call(
      io: io,
      source: file.read,
      filename: file
    )

    raise e if re_raise
  end

  # SyntaxSuggest.call [Private]
  #
  # Main private interface
  def self.call(source:, filename: DEFAULT_VALUE, terminal: DEFAULT_VALUE, record_dir: DEFAULT_VALUE, timeout: TIMEOUT_DEFAULT, io: $stderr)
    search = nil
    filename = nil if filename == DEFAULT_VALUE
    Timeout.timeout(timeout) do
      record_dir ||= ENV["DEBUG"] ? "tmp" : nil
      search = CodeSearch.new(source, record_dir: record_dir).call
    end

    blocks = search.invalid_blocks
    DisplayInvalidBlocks.new(
      io: io,
      blocks: blocks,
      filename: filename,
      terminal: terminal,
      code_lines: search.code_lines
    ).call
  rescue Timeout::Error => e
    io.puts "Search timed out SYNTAX_SUGGEST_TIMEOUT=#{timeout}, run with SYNTAX_SUGGEST_DEBUG=1 for more info"
    io.puts e.backtrace.first(3).join($/)
  end

  # SyntaxSuggest.record_dir [Private]
  #
  # Used to generate a unique directory to record
  # search steps for debugging
  def self.record_dir(dir)
    time = Time.now.strftime("%Y-%m-%d-%H-%M-%s-%N")
    dir = Pathname(dir)
    dir.join(time).tap { |path|
      path.mkpath
      alias_dir = dir.join("last")
      FileUtils.rm_rf(alias_dir) if alias_dir.exist?
      FileUtils.ln_sf(time, alias_dir)
    }
  end

  # SyntaxSuggest.valid_without? [Private]
  #
  # This will tell you if the `code_lines` would be valid
  # if you removed the `without_lines`. In short it's a
  # way to detect if we've found the lines with syntax errors
  # in our document yet.
  #
  #   code_lines = [
  #     CodeLine.new(line: "def foo\n",   index: 0)
  #     CodeLine.new(line: "  def bar\n", index: 1)
  #     CodeLine.new(line: "end\n",       index: 2)
  #   ]
  #
  #   SyntaxSuggest.valid_without?(
  #     without_lines: code_lines[1],
  #     code_lines: code_lines
  #   )                                    # => true
  #
  #   SyntaxSuggest.valid?(code_lines) # => false
  def self.valid_without?(without_lines:, code_lines:)
    lines = code_lines - Array(without_lines).flatten

    lines.empty? || valid?(lines)
  end

  # SyntaxSuggest.invalid? [Private]
  #
  # Opposite of `SyntaxSuggest.valid?`
  if defined?(Prism)
    def self.invalid?(source)
      source = source.join if source.is_a?(Array)
      source = source.to_s

      Prism.parse(source).failure?
    end
  else
    def self.invalid?(source)
      source = source.join if source.is_a?(Array)
      source = source.to_s

      Ripper.new(source).tap(&:parse).error?
    end
  end

  # SyntaxSuggest.valid? [Private]
  #
  # Returns truthy if a given input source is valid syntax
  #
  #   SyntaxSuggest.valid?(<<~EOM) # => true
  #     def foo
  #     end
  #   EOM
  #
  #   SyntaxSuggest.valid?(<<~EOM) # => false
  #     def foo
  #       def bar # Syntax error here
  #     end
  #   EOM
  #
  # You can also pass in an array of lines and they'll be
  # joined before evaluating
  #
  #   SyntaxSuggest.valid?(
  #     [
  #       "def foo\n",
  #       "end\n"
  #     ]
  #   ) # => true
  #
  #   SyntaxSuggest.valid?(
  #     [
  #       "def foo\n",
  #       "  def bar\n", # Syntax error here
  #       "end\n"
  #     ]
  #   ) # => false
  #
  # As an FYI the CodeLine class instances respond to `to_s`
  # so passing a CodeLine in as an object or as an array
  # will convert it to it's code representation.
  def self.valid?(source)
    !invalid?(source)
  end
end

# Integration
# frozen_string_literal: true

require "pathname"
require "optparse"

module SyntaxSuggest
  # All the logic of the exe/syntax_suggest CLI in one handy spot
  #
  #   Cli.new(argv: ["--help"]).call
  #   Cli.new(argv: ["<path/to/file>.rb"]).call
  #   Cli.new(argv: ["<path/to/file>.rb", "--record=tmp"]).call
  #   Cli.new(argv: ["<path/to/file>.rb", "--terminal"]).call
  #
  class Cli
    attr_accessor :options

    # ARGV is Everything passed to the executable, does not include executable name
    #
    # All other intputs are dependency injection for testing
    def initialize(argv:, exit_obj: Kernel, io: $stdout, env: ENV)
      @options = {}
      @parser = nil
      options[:record_dir] = env["SYNTAX_SUGGEST_RECORD_DIR"]
      options[:record_dir] = "tmp" if env["DEBUG"]
      options[:terminal] = SyntaxSuggest::DEFAULT_VALUE

      @io = io
      @argv = argv
      @exit_obj = exit_obj
    end

    def call
      if @argv.empty?
        # Display help if raw command
        parser.parse! %w[--help]
        return
      else
        # Mutates @argv
        parse
        return if options[:exit]
      end

      file_name = @argv.first
      if file_name.nil?
        @io.puts "No file given"
        @exit_obj.exit(1)
        return
      end

      file = Pathname(file_name)
      if !file.exist?
        @io.puts "file not found: #{file.expand_path} "
        @exit_obj.exit(1)
        return
      end

      @io.puts "Record dir: #{options[:record_dir]}" if options[:record_dir]

      display = SyntaxSuggest.call(
        io: @io,
        source: file.read,
        filename: file.expand_path,
        terminal: options.fetch(:terminal, SyntaxSuggest::DEFAULT_VALUE),
        record_dir: options[:record_dir]
      )

      if display.document_ok?
        @io.puts "Syntax OK"
        @exit_obj.exit(0)
      else
        @exit_obj.exit(1)
      end
    end

    def parse
      parser.parse!(@argv)

      self
    end

    def parser
      @parser ||= OptionParser.new do |opts|
        opts.banner = <<~EOM
          Usage: syntax_suggest <file> [options]

          Parses a ruby source file and searches for syntax error(s) such as
          unexpected `end', expecting end-of-input.

          Example:

            $ syntax_suggest dog.rb

            # ...

              > 10  defdog
              > 15  end

          ENV options:

            SYNTAX_SUGGEST_RECORD_DIR=<dir>

            Records the steps used to search for a syntax error
            to the given directory

          Options:
        EOM

        opts.version = SyntaxSuggest::VERSION

        opts.on("--help", "Help - displays this message") do |v|
          @io.puts opts
          options[:exit] = true
          @exit_obj.exit
        end

        opts.on("--record <dir>", "Records the steps used to search for a syntax error to the given directory") do |v|
          options[:record_dir] = v
        end

        opts.on("--terminal", "Enable terminal highlighting") do |v|
          options[:terminal] = true
        end

        opts.on("--no-terminal", "Disable terminal highlighting") do |v|
          options[:terminal] = false
        end
      end
    end
  end
end

# Core logic
# frozen_string_literal: true

module SyntaxSuggest
  # Searches code for a syntax error
  #
  # There are three main phases in the algorithm:
  #
  # 1. Sanitize/format input source
  # 2. Search for invalid blocks
  # 3. Format invalid blocks into something meaninful
  #
  # This class handles the part.
  #
  # The bulk of the heavy lifting is done in:
  #
  #  - CodeFrontier (Holds information for generating blocks and determining if we can stop searching)
  #  - ParseBlocksFromLine (Creates blocks into the frontier)
  #  - BlockExpand (Expands existing blocks to search more code)
  #
  # ## Syntax error detection
  #
  # When the frontier holds the syntax error, we can stop searching
  #
  #   search = CodeSearch.new(<<~EOM)
  #     def dog
  #       def lol
  #     end
  #   EOM
  #
  #   search.call
  #
  #   search.invalid_blocks.map(&:to_s) # =>
  #   # => ["def lol\n"]
  #
  class CodeSearch
    private

    attr_reader :frontier

    public

    attr_reader :invalid_blocks, :record_dir, :code_lines

    def initialize(source, record_dir: DEFAULT_VALUE)
      record_dir = if record_dir == DEFAULT_VALUE
        (ENV["SYNTAX_SUGGEST_RECORD_DIR"] || ENV["SYNTAX_SUGGEST_DEBUG"]) ? "tmp" : nil
      else
        record_dir
      end

      if record_dir
        @record_dir = SyntaxSuggest.record_dir(record_dir)
        @write_count = 0
      end

      @tick = 0
      @source = source
      @name_tick = Hash.new { |hash, k| hash[k] = 0 }
      @invalid_blocks = []

      @code_lines = CleanDocument.new(source: source).call.lines

      @frontier = CodeFrontier.new(code_lines: @code_lines)
      @block_expand = BlockExpand.new(code_lines: @code_lines)
      @parse_blocks_from_indent_line = ParseBlocksFromIndentLine.new(code_lines: @code_lines)
    end

    # Used for debugging
    def record(block:, name: "record")
      return unless @record_dir
      @name_tick[name] += 1
      filename = "#{@write_count += 1}-#{name}-#{@name_tick[name]}-(#{block.starts_at}__#{block.ends_at}).txt"
      if ENV["SYNTAX_SUGGEST_DEBUG"]
        puts "\n\n==== #{filename} ===="
        puts "\n```#{block.starts_at}..#{block.ends_at}"
        puts block
        puts "```"
        puts "  block indent:      #{block.current_indent}"
      end
      @record_dir.join(filename).open(mode: "a") do |f|
        document = DisplayCodeWithLineNumbers.new(
          lines: @code_lines.select(&:visible?),
          terminal: false,
          highlight_lines: block.lines
        ).call

        f.write("    Block lines: #{block.starts_at..block.ends_at} (#{name}) \n\n#{document}")
      end
    end

    def push(block, name:)
      record(block: block, name: name)

      block.mark_invisible if block.valid?
      frontier << block
    end

    # Parses the most indented lines into blocks that are marked
    # and added to the frontier
    def create_blocks_from_untracked_lines
      max_indent = frontier.next_indent_line&.indent

      while (line = frontier.next_indent_line) && (line.indent == max_indent)
        @parse_blocks_from_indent_line.each_neighbor_block(frontier.next_indent_line) do |block|
          push(block, name: "add")
        end
      end
    end

    # Given an already existing block in the frontier, expand it to see
    # if it contains our invalid syntax
    def expand_existing
      block = frontier.pop
      return unless block

      record(block: block, name: "before-expand")

      block = @block_expand.call(block)
      push(block, name: "expand")
    end

    # Main search loop
    def call
      until frontier.holds_all_syntax_errors?
        @tick += 1

        if frontier.expand?
          expand_existing
        else
          create_blocks_from_untracked_lines
        end
      end

      @invalid_blocks.concat(frontier.detect_invalid_blocks)
      @invalid_blocks.sort_by! { |block| block.starts_at }
      self
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # The main function of the frontier is to hold the edges of our search and to
  # evaluate when we can stop searching.

  # There are three main phases in the algorithm:
  #
  # 1. Sanitize/format input source
  # 2. Search for invalid blocks
  # 3. Format invalid blocks into something meaningful
  #
  # The Code frontier is a critical part of the second step
  #
  # ## Knowing where we've been
  #
  # Once a code block is generated it is added onto the frontier. Then it will be
  # sorted by indentation and frontier can be filtered. Large blocks that fully enclose a
  # smaller block will cause the smaller block to be evicted.
  #
  #   CodeFrontier#<<(block) # Adds block to frontier
  #   CodeFrontier#pop # Removes block from frontier
  #
  # ## Knowing where we can go
  #
  # Internally the frontier keeps track of "unvisited" lines which are exposed via `next_indent_line`
  # when called, this method returns, a line of code with the highest indentation.
  #
  # The returned line of code can be used to build a CodeBlock and then that code block
  # is added back to the frontier. Then, the lines are removed from the
  # "unvisited" so we don't double-create the same block.
  #
  #   CodeFrontier#next_indent_line # Shows next line
  #   CodeFrontier#register_indent_block(block) # Removes lines from unvisited
  #
  # ## Knowing when to stop
  #
  # The frontier knows how to check the entire document for a syntax error. When blocks
  # are added onto the frontier, they're removed from the document. When all code containing
  # syntax errors has been added to the frontier, the document will be parsable without a
  # syntax error and the search can stop.
  #
  #   CodeFrontier#holds_all_syntax_errors? # Returns true when frontier holds all syntax errors
  #
  # ## Filtering false positives
  #
  # Once the search is completed, the frontier may have multiple blocks that do not contain
  # the syntax error. To limit the result to the smallest subset of "invalid blocks" call:
  #
  #   CodeFrontier#detect_invalid_blocks
  #
  class CodeFrontier
    def initialize(code_lines:, unvisited: UnvisitedLines.new(code_lines: code_lines))
      @code_lines = code_lines
      @unvisited = unvisited
      @queue = PriorityEngulfQueue.new

      @check_next = true
    end

    def count
      @queue.length
    end

    # Performance optimization
    #
    # Parsing with ripper is expensive
    # If we know we don't have any blocks with invalid
    # syntax, then we know we cannot have found
    # the incorrect syntax yet.
    #
    # When an invalid block is added onto the frontier
    # check document state
    private def can_skip_check?
      check_next = @check_next
      @check_next = false

      if check_next
        false
      else
        true
      end
    end

    # Returns true if the document is valid with all lines
    # removed. By default it checks all blocks in present in
    # the frontier array, but can be used for arbitrary arrays
    # of codeblocks as well
    def holds_all_syntax_errors?(block_array = @queue, can_cache: true)
      return false if can_cache && can_skip_check?

      without_lines = block_array.to_a.flat_map do |block|
        block.lines
      end

      SyntaxSuggest.valid_without?(
        without_lines: without_lines,
        code_lines: @code_lines
      )
    end

    # Returns a code block with the largest indentation possible
    def pop
      @queue.pop
    end

    def next_indent_line
      @unvisited.peek
    end

    def expand?
      return false if @queue.empty?
      return true if @unvisited.empty?

      frontier_indent = @queue.peek.current_indent
      unvisited_indent = next_indent_line.indent

      if ENV["SYNTAX_SUGGEST_DEBUG"]
        puts "```"
        puts @queue.peek
        puts "```"
        puts "  @frontier indent:  #{frontier_indent}"
        puts "  @unvisited indent: #{unvisited_indent}"
      end

      # Expand all blocks before moving to unvisited lines
      frontier_indent >= unvisited_indent
    end

    # Keeps track of what lines have been added to blocks and which are not yet
    # visited.
    def register_indent_block(block)
      @unvisited.visit_block(block)
      self
    end

    # When one element fully encapsulates another we remove the smaller
    # block from the frontier. This prevents double expansions and all-around
    # weird behavior. However this guarantee is quite expensive to maintain
    def register_engulf_block(block)
    end

    # Add a block to the frontier
    #
    # This method ensures the frontier always remains sorted (in indentation order)
    # and that each code block's lines are removed from the indentation hash so we
    # don't re-evaluate the same line multiple times.
    def <<(block)
      @unvisited.visit_block(block)

      @queue.push(block)

      @check_next = true if block.invalid?

      self
    end

    # Example:
    #
    #   combination([:a, :b, :c, :d])
    #   # => [[:a], [:b], [:c], [:d], [:a, :b], [:a, :c], [:a, :d], [:b, :c], [:b, :d], [:c, :d], [:a, :b, :c], [:a, :b, :d], [:a, :c, :d], [:b, :c, :d], [:a, :b, :c, :d]]
    def self.combination(array)
      guesses = []
      1.upto(array.length).each do |size|
        guesses.concat(array.combination(size).to_a)
      end
      guesses
    end

    # Given that we know our syntax error exists somewhere in our frontier, we want to find
    # the smallest possible set of blocks that contain all the syntax errors
    def detect_invalid_blocks
      self.class.combination(@queue.to_a.select(&:invalid?)).detect do |block_array|
        holds_all_syntax_errors?(block_array, can_cache: false)
      end || []
    end
  end
end
# frozen_string_literal: true
# frozen_string_literal: true

module SyntaxSuggest
  # Find mis-matched syntax based on lexical count
  #
  # Used for detecting missing pairs of elements
  # each keyword needs an end, each '{' needs a '}'
  # etc.
  #
  # Example:
  #
  #   left_right = LeftRightLexCount.new
  #   left_right.count_kw
  #   left_right.missing.first
  #   # => "end"
  #
  #   left_right = LeftRightLexCount.new
  #   source = "{ a: b, c: d" # Note missing '}'
  #   LexAll.new(source: source).each do |lex|
  #     left_right.count_lex(lex)
  #   end
  #   left_right.missing.first
  #   # => "}"
  class LeftRightLexCount
    def initialize
      @kw_count = 0
      @end_count = 0

      @count_for_char = {
        "{" => 0,
        "}" => 0,
        "[" => 0,
        "]" => 0,
        "(" => 0,
        ")" => 0,
        "|" => 0
      }
    end

    def count_kw
      @kw_count += 1
    end

    def count_end
      @end_count += 1
    end

    # Count source code characters
    #
    # Example:
    #
    #   left_right = LeftRightLexCount.new
    #   left_right.count_lex(LexValue.new(1, :on_lbrace, "{", Ripper::EXPR_BEG))
    #   left_right.count_for_char("{")
    #   # => 1
    #   left_right.count_for_char("}")
    #   # => 0
    def count_lex(lex)
      case lex.type
      when :on_tstring_content
        # ^^^
        # Means it's a string or a symbol `"{"` rather than being
        # part of a data structure (like a hash) `{ a: b }`
        # ignore it.
      when :on_words_beg, :on_symbos_beg, :on_qwords_beg,
           :on_qsymbols_beg, :on_regexp_beg, :on_tstring_beg
        # ^^^
        # Handle shorthand syntaxes like `%Q{ i am a string }`
        #
        # The start token will be the full thing `%Q{` but we
        # need to count it as if it's a `{`. Any token
        # can be used
        char = lex.token[-1]
        @count_for_char[char] += 1 if @count_for_char.key?(char)
      when :on_embexpr_beg
        # ^^^
        # Embedded string expressions like `"#{foo} <-embed"`
        # are parsed with chars:
        #
        # `#{` as :on_embexpr_beg
        #  `}` as :on_embexpr_end
        #
        # We cannot ignore both :on_emb_expr_beg and :on_embexpr_end
        # because sometimes the lexer thinks something is an embed
        # string end, when it is not like `lol = }` (no clue why).
        #
        # When we see `#{` count it as a `{` or we will
        # have a mis-match count.
        #
        case lex.token
        when "\#{"
          @count_for_char["{"] += 1
        end
      else
        @end_count += 1 if lex.is_end?
        @kw_count += 1 if lex.is_kw?
        @count_for_char[lex.token] += 1 if @count_for_char.key?(lex.token)
      end
    end

    def count_for_char(char)
      @count_for_char[char]
    end

    # Returns an array of missing syntax characters
    # or `"end"` or `"keyword"`
    #
    #   left_right.missing
    #   # => ["}"]
    def missing
      out = missing_pairs
      out << missing_pipe
      out << missing_keyword_end
      out.compact!
      out
    end

    PAIRS = {
      "{" => "}",
      "[" => "]",
      "(" => ")"
    }.freeze

    # Opening characters like `{` need closing characters # like `}`.
    #
    # When a mis-match count is detected, suggest the
    # missing member.
    #
    # For example if there are 3 `}` and only two `{`
    # return `"{"`
    private def missing_pairs
      PAIRS.map do |(left, right)|
        case @count_for_char[left] <=> @count_for_char[right]
        when 1
          right
        when 0
          nil
        when -1
          left
        end
      end
    end

    # Keywords need ends and ends need keywords
    #
    # If we have more keywords, there's a missing `end`
    # if we have more `end`-s, there's a missing keyword
    private def missing_keyword_end
      case @kw_count <=> @end_count
      when 1
        "end"
      when 0
        nil
      when -1
        "keyword"
      end
    end

    # Pipes come in pairs.
    # If there's an odd number of pipes then we
    # are missing one
    private def missing_pipe
      if @count_for_char["|"].odd?
        "|"
      end
    end
  end
end

if !SyntaxSuggest.use_prism_parser?
# frozen_string_literal: true

module SyntaxSuggest
  # Capture parse errors from Ripper
  #
  # Prism returns the errors with their messages, but Ripper
  # does not. To get them we must make a custom subclass.
  #
  # Example:
  #
  #   puts RipperErrors.new(" def foo").call.errors
  #   # => ["syntax error, unexpected end-of-input, expecting ';' or '\\n'"]
  class RipperErrors < Ripper
    attr_reader :errors

    # Comes from ripper, called
    # on every parse error, msg
    # is a string
    def on_parse_error(msg)
      @errors ||= []
      @errors << msg
    end

    alias_method :on_alias_error, :on_parse_error
    alias_method :on_assign_error, :on_parse_error
    alias_method :on_class_name_error, :on_parse_error
    alias_method :on_param_error, :on_parse_error
    alias_method :compile_error, :on_parse_error

    def call
      @run_once ||= begin
        @errors = []
        parse
        true
      end
      self
    end
  end
end
end

module SyntaxSuggest
  class GetParseErrors
    def self.errors(source)
      if SyntaxSuggest.use_prism_parser?
        Prism.parse(source).errors.map(&:message)
      else
        RipperErrors.new(source).call.errors
      end
    end
  end

  # Explains syntax errors based on their source
  #
  # example:
  #
  #   source = "def foo; puts 'lol'" # Note missing end
  #   explain ExplainSyntax.new(
  #     code_lines: CodeLine.from_source(source)
  #   ).call
  #   explain.errors.first
  #   # => "Unmatched keyword, missing `end' ?"
  #
  # When the error cannot be determined by lexical counting
  # then the parser is run against the input and the raw
  # errors are returned.
  #
  # Example:
  #
  #   source = "1 * " # Note missing a second number
  #   explain ExplainSyntax.new(
  #     code_lines: CodeLine.from_source(source)
  #   ).call
  #   explain.errors.first
  #   # => "syntax error, unexpected end-of-input"
  class ExplainSyntax
    INVERSE = {
      "{" => "}",
      "}" => "{",
      "[" => "]",
      "]" => "[",
      "(" => ")",
      ")" => "(",
      "|" => "|"
    }.freeze

    def initialize(code_lines:)
      @code_lines = code_lines
      @left_right = LeftRightLexCount.new
      @missing = nil
    end

    def call
      @code_lines.each do |line|
        line.lex.each do |lex|
          @left_right.count_lex(lex)
        end
      end

      self
    end

    # Returns an array of missing elements
    #
    # For example this:
    #
    #   ExplainSyntax.new(code_lines: lines).missing
    #   # => ["}"]
    #
    # Would indicate that the source is missing
    # a `}` character in the source code
    def missing
      @missing ||= @left_right.missing
    end

    # Converts a missing string to
    # an human understandable explanation.
    #
    # Example:
    #
    #   explain.why("}")
    #   # => "Unmatched `{', missing `}' ?"
    #
    def why(miss)
      case miss
      when "keyword"
        "Unmatched `end', missing keyword (`do', `def`, `if`, etc.) ?"
      when "end"
        "Unmatched keyword, missing `end' ?"
      else
        inverse = INVERSE.fetch(miss) {
          raise "Unknown explain syntax char or key: #{miss.inspect}"
        }
        "Unmatched `#{inverse}', missing `#{miss}' ?"
      end
    end

    # Returns an array of syntax error messages
    #
    # If no missing pairs are found it falls back
    # on the original error messages
    def errors
      if missing.empty?
        return GetParseErrors.errors(@code_lines.map(&:original).join).uniq
      end

      missing.map { |miss| why(miss) }
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # Parses and sanitizes source into a lexically aware document
  #
  # Internally the document is represented by an array with each
  # index containing a CodeLine correlating to a line from the source code.
  #
  # There are three main phases in the algorithm:
  #
  # 1. Sanitize/format input source
  # 2. Search for invalid blocks
  # 3. Format invalid blocks into something meaningful
  #
  # This class handles the first part.
  #
  # The reason this class exists is to format input source
  # for better/easier/cleaner exploration.
  #
  # The CodeSearch class operates at the line level so
  # we must be careful to not introduce lines that look
  # valid by themselves, but when removed will trigger syntax errors
  # or strange behavior.
  #
  # ## Join Trailing slashes
  #
  # Code with a trailing slash is logically treated as a single line:
  #
  #     1 it "code can be split" \
  #     2    "across multiple lines" do
  #
  # In this case removing line 2 would add a syntax error. We get around
  # this by internally joining the two lines into a single "line" object
  #
  # ## Logically Consecutive lines
  #
  # Code that can be broken over multiple
  # lines such as method calls are on different lines:
  #
  #     1 User.
  #     2   where(name: "schneems").
  #     3   first
  #
  # Removing line 2 can introduce a syntax error. To fix this, all lines
  # are joined into one.
  #
  # ## Heredocs
  #
  # A heredoc is an way of defining a multi-line string. They can cause many
  # problems. If left as a single line, the parser would try to parse the contents
  # as ruby code rather than as a string. Even without this problem, we still
  # hit an issue with indentation:
  #
  #    1 foo = <<~HEREDOC
  #    2  "Be yourself; everyone else is already taken.""
  #    3    ― Oscar Wilde
  #    4      puts "I look like ruby code" # but i'm still a heredoc
  #    5 HEREDOC
  #
  # If we didn't join these lines then our algorithm would think that line 4
  # is separate from the rest, has a higher indentation, then look at it first
  # and remove it.
  #
  # If the code evaluates line 5 by itself it will think line 5 is a constant,
  # remove it, and introduce a syntax errror.
  #
  # All of these problems are fixed by joining the whole heredoc into a single
  # line.
  #
  # ## Comments and whitespace
  #
  # Comments can throw off the way the lexer tells us that the line
  # logically belongs with the next line. This is valid ruby but
  # results in a different lex output than before:
  #
  #     1 User.
  #     2   where(name: "schneems").
  #     3   # Comment here
  #     4   first
  #
  # To handle this we can replace comment lines with empty lines
  # and then re-lex the source. This removal and re-lexing preserves
  # line index and document size, but generates an easier to work with
  # document.
  #
  class CleanDocument
    def initialize(source:)
      lines = clean_sweep(source: source)
      @document = CodeLine.from_source(lines.join, lines: lines)
    end

    # Call all of the document "cleaners"
    # and return self
    def call
      join_trailing_slash!
      join_consecutive!
      join_heredoc!

      self
    end

    # Return an array of CodeLines in the
    # document
    def lines
      @document
    end

    # Renders the document back to a string
    def to_s
      @document.join
    end

    # Remove comments
    #
    # replace with empty newlines
    #
    #     source = <<~'EOM'
    #       # Comment 1
    #       puts "hello"
    #       # Comment 2
    #       puts "world"
    #     EOM
    #
    #     lines = CleanDocument.new(source: source).lines
    #     expect(lines[0].to_s).to eq("\n")
    #     expect(lines[1].to_s).to eq("puts "hello")
    #     expect(lines[2].to_s).to eq("\n")
    #     expect(lines[3].to_s).to eq("puts "world")
    #
    # Important: This must be done before lexing.
    #
    # After this change is made, we lex the document because
    # removing comments can change how the doc is parsed.
    #
    # For example:
    #
    #     values = LexAll.new(source: <<~EOM))
    #       User.
    #         # comment
    #         where(name: 'schneems')
    #     EOM
    #     expect(
    #       values.count {|v| v.type == :on_ignored_nl}
    #     ).to eq(1)
    #
    # After the comment is removed:
    #
    #     values = LexAll.new(source: <<~EOM))
    #       User.
    #
    #         where(name: 'schneems')
    #     EOM
    #     expect(
    #      values.count {|v| v.type == :on_ignored_nl}
    #    ).to eq(2)
    #
    def clean_sweep(source:)
      # Match comments, but not HEREDOC strings with #{variable} interpolation
      # https://rubular.com/r/HPwtW9OYxKUHXQ
      source.lines.map do |line|
        if line.match?(/^\s*#([^{].*|)$/)
          $/
        else
          line
        end
      end
    end

    # Smushes all heredoc lines into one line
    #
    #     source = <<~'EOM'
    #       foo = <<~HEREDOC
    #          lol
    #          hehehe
    #       HEREDOC
    #     EOM
    #
    #     lines = CleanDocument.new(source: source).join_heredoc!.lines
    #     expect(lines[0].to_s).to eq(source)
    #     expect(lines[1].to_s).to eq("")
    def join_heredoc!
      start_index_stack = []
      heredoc_beg_end_index = []
      lines.each do |line|
        line.lex.each do |lex_value|
          case lex_value.type
          when :on_heredoc_beg
            start_index_stack << line.index
          when :on_heredoc_end
            start_index = start_index_stack.pop
            end_index = line.index
            heredoc_beg_end_index << [start_index, end_index]
          end
        end
      end

      heredoc_groups = heredoc_beg_end_index.map { |start_index, end_index| @document[start_index..end_index] }

      join_groups(heredoc_groups)
      self
    end

    # Smushes logically "consecutive" lines
    #
    #     source = <<~'EOM'
    #       User.
    #         where(name: 'schneems').
    #         first
    #     EOM
    #
    #     lines = CleanDocument.new(source: source).join_consecutive!.lines
    #     expect(lines[0].to_s).to eq(source)
    #     expect(lines[1].to_s).to eq("")
    #
    # The one known case this doesn't handle is:
    #
    #     Ripper.lex <<~EOM
    #       a &&
    #        b ||
    #        c
    #     EOM
    #
    # For some reason this introduces `on_ignore_newline` but with BEG type
    #
    def join_consecutive!
      consecutive_groups = @document.select(&:ignore_newline_not_beg?).map do |code_line|
        take_while_including(code_line.index..) do |line|
          line.ignore_newline_not_beg?
        end
      end

      join_groups(consecutive_groups)
      self
    end

    # Join lines with a trailing slash
    #
    #     source = <<~'EOM'
    #       it "code can be split" \
    #          "across multiple lines" do
    #     EOM
    #
    #     lines = CleanDocument.new(source: source).join_consecutive!.lines
    #     expect(lines[0].to_s).to eq(source)
    #     expect(lines[1].to_s).to eq("")
    def join_trailing_slash!
      trailing_groups = @document.select(&:trailing_slash?).map do |code_line|
        take_while_including(code_line.index..) { |x| x.trailing_slash? }
      end
      join_groups(trailing_groups)
      self
    end

    # Helper method for joining "groups" of lines
    #
    # Input is expected to be type Array<Array<CodeLine>>
    #
    # The outer array holds the various "groups" while the
    # inner array holds code lines.
    #
    # All code lines are "joined" into the first line in
    # their group.
    #
    # To preserve document size, empty lines are placed
    # in the place of the lines that were "joined"
    def join_groups(groups)
      groups.each do |lines|
        line = lines.first

        # Handle the case of multiple groups in a row
        # if one is already replaced, move on
        next if @document[line.index].empty?

        # Join group into the first line
        @document[line.index] = CodeLine.new(
          lex: lines.map(&:lex).flatten,
          line: lines.join,
          index: line.index
        )

        # Hide the rest of the lines
        lines[1..].each do |line|
          # The above lines already have newlines in them, if add more
          # then there will be double newline, use an empty line instead
          @document[line.index] = CodeLine.new(line: "", index: line.index, lex: [])
        end
      end
      self
    end

    # Helper method for grabbing elements from document
    #
    # Like `take_while` except when it stops
    # iterating, it also returns the line
    # that caused it to stop
    def take_while_including(range = 0..)
      take_next_and_stop = false
      @document[range].take_while do |line|
        next if take_next_and_stop

        take_next_and_stop = !(yield line)
        true
      end
    end
  end
end

# Helpers
# frozen_string_literal: true

module SyntaxSuggest
  # Ripper.lex is not guaranteed to lex the entire source document
  #
  # This class guarantees the whole document is lex-ed by iteratively
  # lexing the document where ripper stopped.
  #
  # Prism likely doesn't have the same problem. Once ripper support is removed
  # we can likely reduce the complexity here if not remove the whole concept.
  #
  # Example usage:
  #
  #   lex = LexAll.new(source: source)
  #   lex.each do |value|
  #     puts value.line
  #   end
  class LexAll
    include Enumerable

    def initialize(source:, source_lines: nil)
      @lex = self.class.lex(source, 1)
      lineno = @lex.last[0][0] + 1
      source_lines ||= source.lines
      last_lineno = source_lines.length

      until lineno >= last_lineno
        lines = source_lines[lineno..]

        @lex.concat(
          self.class.lex(lines.join, lineno + 1)
        )

        lineno = @lex.last[0].first + 1
      end

      last_lex = nil
      @lex.map! { |elem|
        last_lex = LexValue.new(elem[0].first, elem[1], elem[2], elem[3], last_lex)
      }
    end

    if SyntaxSuggest.use_prism_parser?
      def self.lex(source, line_number)
        Prism.lex_compat(source, line: line_number).value.sort_by { |values| values[0] }
      end
    else
      def self.lex(source, line_number)
        Ripper::Lexer.new(source, "-", line_number).parse.sort_by(&:pos)
      end
    end

    def to_a
      @lex
    end

    def each
      return @lex.each unless block_given?
      @lex.each do |x|
        yield x
      end
    end

    def [](index)
      @lex[index]
    end

    def last
      @lex.last
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # Value object for accessing lex values
  #
  # This lex:
  #
  #   [1, 0], :on_ident, "describe", CMDARG
  #
  # Would translate into:
  #
  #  lex.line # => 1
  #  lex.type # => :on_indent
  #  lex.token # => "describe"
  class LexValue
    attr_reader :line, :type, :token, :state

    def initialize(line, type, token, state, last_lex = nil)
      @line = line
      @type = type
      @token = token
      @state = state

      set_kw_end(last_lex)
    end

    private def set_kw_end(last_lex)
      @is_end = false
      @is_kw = false
      return if type != :on_kw

      return if last_lex && last_lex.fname? # https://github.com/ruby/ruby/commit/776759e300e4659bb7468e2b97c8c2d4359a2953

      case token
      when "if", "unless", "while", "until"
        # Only count if/unless when it's not a "trailing" if/unless
        # https://github.com/ruby/ruby/blob/06b44f819eb7b5ede1ff69cecb25682b56a1d60c/lib/irb/ruby-lex.rb#L374-L375
        @is_kw = true unless expr_label?
      when "def", "case", "for", "begin", "class", "module", "do"
        @is_kw = true
      when "end"
        @is_end = true
      end
    end

    def fname?
      state.allbits?(Ripper::EXPR_FNAME)
    end

    def ignore_newline?
      type == :on_ignored_nl
    end

    def is_end?
      @is_end
    end

    def is_kw?
      @is_kw
    end

    def expr_beg?
      state.anybits?(Ripper::EXPR_BEG)
    end

    def expr_label?
      state.allbits?(Ripper::EXPR_LABEL)
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # Represents a single line of code of a given source file
  #
  # This object contains metadata about the line such as
  # amount of indentation, if it is empty or not, and
  # lexical data, such as if it has an `end` or a keyword
  # in it.
  #
  # Visibility of lines can be toggled off. Marking a line as invisible
  # indicates that it should not be used for syntax checks.
  # It's functionally the same as commenting it out.
  #
  # Example:
  #
  #   line = CodeLine.from_source("def foo\n").first
  #   line.number => 1
  #   line.empty? # => false
  #   line.visible? # => true
  #   line.mark_invisible
  #   line.visible? # => false
  #
  class CodeLine
    TRAILING_SLASH = ("\\" + $/).freeze

    # Returns an array of CodeLine objects
    # from the source string
    def self.from_source(source, lines: nil)
      lines ||= source.lines
      lex_array_for_line = LexAll.new(source: source, source_lines: lines).each_with_object(Hash.new { |h, k| h[k] = [] }) { |lex, hash| hash[lex.line] << lex }
      lines.map.with_index do |line, index|
        CodeLine.new(
          line: line,
          index: index,
          lex: lex_array_for_line[index + 1]
        )
      end
    end

    attr_reader :line, :index, :lex, :line_number, :indent
    def initialize(line:, index:, lex:)
      @lex = lex
      @line = line
      @index = index
      @original = line
      @line_number = @index + 1
      strip_line = line.dup
      strip_line.lstrip!

      @indent = if (@empty = strip_line.empty?)
        line.length - 1 # Newline removed from strip_line is not "whitespace"
      else
        line.length - strip_line.length
      end

      set_kw_end
    end

    # Used for stable sort via indentation level
    #
    # Ruby's sort is not "stable" meaning that when
    # multiple elements have the same value, they are
    # not guaranteed to return in the same order they
    # were put in.
    #
    # So when multiple code lines have the same indentation
    # level, they're sorted by their index value which is unique
    # and consistent.
    #
    # This is mostly needed for consistency of the test suite
    def indent_index
      @indent_index ||= [indent, index]
    end
    alias_method :number, :line_number

    # Returns true if the code line is determined
    # to contain a keyword that matches with an `end`
    #
    # For example: `def`, `do`, `begin`, `ensure`, etc.
    def is_kw?
      @is_kw
    end

    # Returns true if the code line is determined
    # to contain an `end` keyword
    def is_end?
      @is_end
    end

    # Used to hide lines
    #
    # The search alorithm will group lines into blocks
    # then if those blocks are determined to represent
    # valid code they will be hidden
    def mark_invisible
      @line = ""
    end

    # Means the line was marked as "invisible"
    # Confusingly, "empty" lines are visible...they
    # just don't contain any source code other than a newline ("\n").
    def visible?
      !line.empty?
    end

    # Opposite or `visible?` (note: different than `empty?`)
    def hidden?
      !visible?
    end

    # An `empty?` line is one that was originally left
    # empty in the source code, while a "hidden" line
    # is one that we've since marked as "invisible"
    def empty?
      @empty
    end

    # Opposite of `empty?` (note: different than `visible?`)
    def not_empty?
      !empty?
    end

    # Renders the given line
    #
    # Also allows us to represent source code as
    # an array of code lines.
    #
    # When we have an array of code line elements
    # calling `join` on the array will call `to_s`
    # on each element, which essentially converts
    # it back into it's original source string.
    def to_s
      line
    end

    # When the code line is marked invisible
    # we retain the original value of it's line
    # this is useful for debugging and for
    # showing extra context
    #
    # DisplayCodeWithLineNumbers will render
    # all lines given to it, not just visible
    # lines, it uses the original method to
    # obtain them.
    attr_reader :original

    # Comparison operator, needed for equality
    # and sorting
    def <=>(other)
      index <=> other.index
    end

    # [Not stable API]
    #
    # Lines that have a `on_ignored_nl` type token and NOT
    # a `BEG` type seem to be a good proxy for the ability
    # to join multiple lines into one.
    #
    # This predicate method is used to determine when those
    # two criteria have been met.
    #
    # The one known case this doesn't handle is:
    #
    #     Ripper.lex <<~EOM
    #       a &&
    #        b ||
    #        c
    #     EOM
    #
    # For some reason this introduces `on_ignore_newline` but with BEG type
    def ignore_newline_not_beg?
      @ignore_newline_not_beg
    end

    # Determines if the given line has a trailing slash
    #
    #     lines = CodeLine.from_source(<<~EOM)
    #       it "foo" \
    #     EOM
    #     expect(lines.first.trailing_slash?).to eq(true)
    #
    def trailing_slash?
      last = @lex.last

      # Older versions of prism diverged slightly from Ripper in compatibility mode
      case last&.type
      when :on_sp
        last.token == TRAILING_SLASH
      when :on_tstring_end
        true
      else
        false
      end
    end

    # Endless method detection
    #
    # From https://github.com/ruby/irb/commit/826ae909c9c93a2ddca6f9cfcd9c94dbf53d44ab
    # Detecting a "oneliner" seems to need a state machine.
    # This can be done by looking mostly at the "state" (last value):
    #
    #   ENDFN -> BEG (token = '=' ) -> END
    #
    private def set_kw_end
      oneliner_count = 0
      in_oneliner_def = nil

      kw_count = 0
      end_count = 0

      @ignore_newline_not_beg = false
      @lex.each do |lex|
        kw_count += 1 if lex.is_kw?
        end_count += 1 if lex.is_end?

        if lex.type == :on_ignored_nl
          @ignore_newline_not_beg = !lex.expr_beg?
        end

        if in_oneliner_def.nil?
          in_oneliner_def = :ENDFN if lex.state.allbits?(Ripper::EXPR_ENDFN)
        elsif lex.state.allbits?(Ripper::EXPR_ENDFN)
          # Continue
        elsif lex.state.allbits?(Ripper::EXPR_BEG)
          in_oneliner_def = :BODY if lex.token == "="
        elsif lex.state.allbits?(Ripper::EXPR_END)
          # We found an endless method, count it
          oneliner_count += 1 if in_oneliner_def == :BODY

          in_oneliner_def = nil
        else
          in_oneliner_def = nil
        end
      end

      kw_count -= oneliner_count

      @is_kw = (kw_count - end_count) > 0
      @is_end = (end_count - kw_count) > 0
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # Multiple lines form a singular CodeBlock
  #
  # Source code is made of multiple CodeBlocks.
  #
  # Example:
  #
  #   code_block.to_s # =>
  #     #   def foo
  #     #     puts "foo"
  #     #   end
  #
  #   code_block.valid? # => true
  #   code_block.in_valid? # => false
  #
  #
  class CodeBlock
    UNSET = Object.new.freeze
    attr_reader :lines, :starts_at, :ends_at

    def initialize(lines: [])
      @lines = Array(lines)
      @valid = UNSET
      @deleted = false
      @starts_at = @lines.first.number
      @ends_at = @lines.last.number
    end

    def delete
      @deleted = true
    end

    def deleted?
      @deleted
    end

    def visible_lines
      @lines.select(&:visible?).select(&:not_empty?)
    end

    def mark_invisible
      @lines.map(&:mark_invisible)
    end

    def is_end?
      to_s.strip == "end"
    end

    def hidden?
      @lines.all?(&:hidden?)
    end

    # This is used for frontier ordering, we are searching from
    # the largest indentation to the smallest. This allows us to
    # populate an array with multiple code blocks then call `sort!`
    # on it without having to specify the sorting criteria
    def <=>(other)
      out = current_indent <=> other.current_indent
      return out if out != 0

      # Stable sort
      starts_at <=> other.starts_at
    end

    def current_indent
      @current_indent ||= lines.select(&:not_empty?).map(&:indent).min || 0
    end

    def invalid?
      !valid?
    end

    def valid?
      if @valid == UNSET
        # Performance optimization
        #
        # If all the lines were previously hidden
        # and we expand to capture additional empty
        # lines then the result cannot be invalid
        #
        # That means there's no reason to re-check all
        # lines with the parser (which is expensive).
        # Benchmark in commit message
        @valid = if lines.all? { |l| l.hidden? || l.empty? }
          true
        else
          SyntaxSuggest.valid?(lines.map(&:original).join)
        end
      else
        @valid
      end
    end

    def to_s
      @lines.join
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # This class is responsible for taking a code block that exists
  # at a far indentaion and then iteratively increasing the block
  # so that it captures everything within the same indentation block.
  #
  #   def dog
  #     puts "bow"
  #     puts "wow"
  #   end
  #
  # block = BlockExpand.new(code_lines: code_lines)
  #   .call(CodeBlock.new(lines: code_lines[1]))
  #
  # puts block.to_s
  # # => puts "bow"
  #      puts "wow"
  #
  #
  # Once a code block has captured everything at a given indentation level
  # then it will expand to capture surrounding indentation.
  #
  # block = BlockExpand.new(code_lines: code_lines)
  #   .call(block)
  #
  # block.to_s
  # # => def dog
  #        puts "bow"
  #        puts "wow"
  #      end
  #
  class BlockExpand
    def initialize(code_lines:)
      @code_lines = code_lines
    end

    # Main interface. Expand current indentation, before
    # expanding to a lower indentation
    def call(block)
      if (next_block = expand_neighbors(block))
        next_block
      else
        expand_indent(block)
      end
    end

    # Expands code to the next lowest indentation
    #
    # For example:
    #
    #   1 def dog
    #   2   print "dog"
    #   3 end
    #
    # If a block starts on line 2 then it has captured all it's "neighbors" (code at
    # the same indentation or higher). To continue expanding, this block must capture
    # lines one and three which are at a different indentation level.
    #
    # This method allows fully expanded blocks to decrease their indentation level (so
    # they can expand to capture more code up and down). It does this conservatively
    # as there's no undo (currently).
    def expand_indent(block)
      now = AroundBlockScan.new(code_lines: @code_lines, block: block)
        .force_add_hidden
        .stop_after_kw
        .scan_adjacent_indent

      now.lookahead_balance_one_line

      now.code_block
    end

    # A neighbor is code that is at or above the current indent line.
    #
    # First we build a block with all neighbors. If we can't go further
    # then we decrease the indentation threshold and expand via indentation
    # i.e. `expand_indent`
    #
    # Handles two general cases.
    #
    # ## Case #1: Check code inside of methods/classes/etc.
    #
    # It's important to note, that not everything in a given indentation level can be parsed
    # as valid code even if it's part of valid code. For example:
    #
    #   1 hash = {
    #   2   name: "richard",
    #   3   dog: "cinco",
    #   4 }
    #
    # In this case lines 2 and 3 will be neighbors, but they're invalid until `expand_indent`
    # is called on them.
    #
    # When we are adding code within a method or class (at the same indentation level),
    # use the empty lines to denote the programmer intended logical chunks.
    # Stop and check each one. For example:
    #
    #   1 def dog
    #   2   print "dog"
    #   3
    #   4   hash = {
    #   5 end
    #
    # If we did not stop parsing at empty newlines then the block might mistakenly grab all
    # the contents (lines 2, 3, and 4) and report them as being problems, instead of only
    # line 4.
    #
    # ## Case #2: Expand/grab other logical blocks
    #
    # Once the search algorithm has converted all lines into blocks at a given indentation
    # it will then `expand_indent`. Once the blocks that generates are expanded as neighbors
    # we then begin seeing neighbors being other logical blocks i.e. a block's neighbors
    # may be another method or class (something with keywords/ends).
    #
    # For example:
    #
    #   1 def bark
    #   2
    #   3 end
    #   4
    #   5 def sit
    #   6 end
    #
    # In this case if lines 4, 5, and 6 are in a block when it tries to expand neighbors
    # it will expand up. If it stops after line 2 or 3 it may cause problems since there's a
    # valid kw/end pair, but the block will be checked without it.
    #
    # We try to resolve this edge case with `lookahead_balance_one_line` below.
    def expand_neighbors(block)
      now = AroundBlockScan.new(code_lines: @code_lines, block: block)

      # Initial scan
      now
        .force_add_hidden
        .stop_after_kw
        .scan_neighbors_not_empty

      # Slurp up empties
      now
        .scan_while { |line| line.empty? }

      # If next line is kw and it will balance us, take it
      expanded_lines = now
        .lookahead_balance_one_line
        .lines

      # Don't allocate a block if it won't be used
      #
      # If nothing was taken, return nil to indicate that status
      # used in `def call` to determine if
      # we need to expand up/out (`expand_indent`)
      if block.lines == expanded_lines
        nil
      else
        CodeBlock.new(lines: expanded_lines)
      end
    end

    # Manageable rspec errors
    def inspect
      "#<SyntaxSuggest::CodeBlock:0x0000123843lol >"
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # Mini String IO [Private]
  #
  # Acts like a StringIO with reduced API, but without having to require that
  # class.
  #
  # The original codebase emitted directly to $stderr, but now SyntaxError#detailed_message
  # needs a string output. To accomplish that we kept the original print infrastructure in place and
  # added this class to accumulate the print output into a string.
  class MiniStringIO
    EMPTY_ARG = Object.new

    def initialize(isatty: $stderr.isatty)
      @string = +""
      @isatty = isatty
    end

    attr_reader :isatty
    def puts(value = EMPTY_ARG, **)
      if !value.equal?(EMPTY_ARG)
        @string << value
      end
      @string << $/
    end

    attr_reader :string
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # Holds elements in a priority heap on insert
  #
  # Instead of constantly calling `sort!`, put
  # the element where it belongs the first time
  # around
  #
  # Example:
  #
  #   queue = PriorityQueue.new
  #   queue << 33
  #   queue << 44
  #   queue << 1
  #
  #   puts queue.peek # => 44
  #
  class PriorityQueue
    attr_reader :elements

    def initialize
      @elements = []
    end

    def <<(element)
      @elements << element
      bubble_up(last_index, element)
    end

    def pop
      exchange(0, last_index)
      max = @elements.pop
      bubble_down(0)
      max
    end

    def length
      @elements.length
    end

    def empty?
      @elements.empty?
    end

    def peek
      @elements.first
    end

    def to_a
      @elements
    end

    # Used for testing, extremely not performant
    def sorted
      out = []
      elements = @elements.dup
      while (element = pop)
        out << element
      end
      @elements = elements
      out.reverse
    end

    private def last_index
      @elements.size - 1
    end

    private def bubble_up(index, element)
      return if index <= 0

      parent_index = (index - 1) / 2
      parent = @elements[parent_index]

      return if (parent <=> element) >= 0

      exchange(index, parent_index)
      bubble_up(parent_index, element)
    end

    private def bubble_down(index)
      child_index = (index * 2) + 1

      return if child_index > last_index

      not_the_last_element = child_index < last_index
      left_element = @elements[child_index]
      right_element = @elements[child_index + 1]

      child_index += 1 if not_the_last_element && (right_element <=> left_element) == 1

      return if (@elements[index] <=> @elements[child_index]) >= 0

      exchange(index, child_index)
      bubble_down(child_index)
    end

    def exchange(source, target)
      a = @elements[source]
      b = @elements[target]
      @elements[source] = b
      @elements[target] = a
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # Tracks which lines various code blocks have expanded to
  # and which are still unexplored
  class UnvisitedLines
    def initialize(code_lines:)
      @unvisited = code_lines.sort_by(&:indent_index)
      @visited_lines = {}
      @visited_lines.compare_by_identity
    end

    def empty?
      @unvisited.empty?
    end

    def peek
      @unvisited.last
    end

    def pop
      @unvisited.pop
    end

    def visit_block(block)
      block.lines.each do |line|
        next if @visited_lines[line]
        @visited_lines[line] = true
      end

      while @visited_lines[@unvisited.last]
        @unvisited.pop
      end
    end
  end
end
# frozen_string_literal: true
# frozen_string_literal: true

module SyntaxSuggest
  # Scans up/down from the given block
  #
  # You can try out a change, stash it, or commit it to save for later
  #
  # Example:
  #
  #   scanner = ScanHistory.new(code_lines: code_lines, block: block)
  #   scanner.scan(
  #     up: ->(_, _, _) { true },
  #     down: ->(_, _, _) { true }
  #   )
  #   scanner.changed? # => true
  #   expect(scanner.lines).to eq(code_lines)
  #
  #   scanner.stash_changes
  #
  #   expect(scanner.lines).to_not eq(code_lines)
  class ScanHistory
    attr_reader :before_index, :after_index

    def initialize(code_lines:, block:)
      @code_lines = code_lines
      @history = [block]
      refresh_index
    end

    def commit_if_changed
      if changed?
        @history << CodeBlock.new(lines: @code_lines[before_index..after_index])
      end

      self
    end

    # Discards any changes that have not been committed
    def stash_changes
      refresh_index
      self
    end

    # Discard changes that have not been committed and revert the last commit
    #
    # Cannot revert the first commit
    def revert_last_commit
      if @history.length > 1
        @history.pop
        refresh_index
      end

      self
    end

    def changed?
      @before_index != current.lines.first.index ||
        @after_index != current.lines.last.index
    end

    # Iterates up and down
    #
    # Returns line, kw_count, end_count for each iteration
    def scan(up:, down:)
      kw_count = 0
      end_count = 0

      up_index = before_lines.reverse_each.take_while do |line|
        kw_count += 1 if line.is_kw?
        end_count += 1 if line.is_end?
        up.call(line, kw_count, end_count)
      end.last&.index

      kw_count = 0
      end_count = 0

      down_index = after_lines.each.take_while do |line|
        kw_count += 1 if line.is_kw?
        end_count += 1 if line.is_end?
        down.call(line, kw_count, end_count)
      end.last&.index

      @before_index = if up_index && up_index < @before_index
        up_index
      else
        @before_index
      end

      @after_index = if down_index && down_index > @after_index
        down_index
      else
        @after_index
      end

      self
    end

    def next_up
      return nil if @before_index <= 0

      @code_lines[@before_index - 1]
    end

    def next_down
      return nil if @after_index >= @code_lines.length

      @code_lines[@after_index + 1]
    end

    def lines
      @code_lines[@before_index..@after_index]
    end

    private def before_lines
      @code_lines[0...@before_index] || []
    end

    # Returns an array of all the CodeLines that exist after
    # the currently scanned block
    private def after_lines
      @code_lines[@after_index.next..] || []
    end

    private def current
      @history.last
    end

    private def refresh_index
      @before_index = current.lines.first.index
      @after_index = current.lines.last.index
      self
    end
  end
end

module SyntaxSuggest
  # This class is useful for exploring contents before and after
  # a block
  #
  # It searches above and below the passed in block to match for
  # whatever criteria you give it:
  #
  # Example:
  #
  #   def dog         # 1
  #     puts "bark"   # 2
  #     puts "bark"   # 3
  #   end             # 4
  #
  #   scan = AroundBlockScan.new(
  #     code_lines: code_lines
  #     block: CodeBlock.new(lines: code_lines[1])
  #   )
  #
  #   scan.scan_while { true }
  #
  #   puts scan.before_index # => 0
  #   puts scan.after_index  # => 3
  #
  class AroundBlockScan
    def initialize(code_lines:, block:)
      @code_lines = code_lines
      @orig_indent = block.current_indent

      @stop_after_kw = false
      @force_add_empty = false
      @force_add_hidden = false
      @target_indent = nil

      @scanner = ScanHistory.new(code_lines: code_lines, block: block)
    end

    # When using this flag, `scan_while` will
    # bypass the block it's given and always add a
    # line that responds truthy to `CodeLine#hidden?`
    #
    # Lines are hidden when they've been evaluated by
    # the parser as part of a block and found to contain
    # valid code.
    def force_add_hidden
      @force_add_hidden = true
      self
    end

    # When using this flag, `scan_while` will
    # bypass the block it's given and always add a
    # line that responds truthy to `CodeLine#empty?`
    #
    # Empty lines contain no code, only whitespace such
    # as leading spaces a newline.
    def force_add_empty
      @force_add_empty = true
      self
    end

    # Tells `scan_while` to look for mismatched keyword/end-s
    #
    # When scanning up, if we see more keywords then end-s it will
    # stop. This might happen when scanning outside of a method body.
    # the first scan line up would be a keyword and this setting would
    # trigger a stop.
    #
    # When scanning down, stop if there are more end-s than keywords.
    def stop_after_kw
      @stop_after_kw = true
      self
    end

    # Main work method
    #
    # The scan_while method takes a block that yields lines above and
    # below the block. If the yield returns true, the @before_index
    # or @after_index are modified to include the matched line.
    #
    # In addition to yielding individual lines, the internals of this
    # object give a mini DSL to handle common situations such as
    # stopping if we've found a keyword/end mis-match in one direction
    # or the other.
    def scan_while
      stop_next_up = false
      stop_next_down = false

      @scanner.scan(
        up: ->(line, kw_count, end_count) {
          next false if stop_next_up
          next true if @force_add_hidden && line.hidden?
          next true if @force_add_empty && line.empty?

          if @stop_after_kw && kw_count > end_count
            stop_next_up = true
          end

          yield line
        },
        down: ->(line, kw_count, end_count) {
          next false if stop_next_down
          next true if @force_add_hidden && line.hidden?
          next true if @force_add_empty && line.empty?

          if @stop_after_kw && end_count > kw_count
            stop_next_down = true
          end

          yield line
        }
      )

      self
    end

    # Scanning is intentionally conservative because
    # we have no way of rolling back an aggressive block (at this time)
    #
    # If a block was stopped for some trivial reason, (like an empty line)
    # but the next line would have caused it to be balanced then we
    # can check that condition and grab just one more line either up or
    # down.
    #
    # For example, below if we're scanning up, line 2 might cause
    # the scanning to stop. This is because empty lines might
    # denote logical breaks where the user intended to chunk code
    # which is a good place to stop and check validity. Unfortunately
    # it also means we might have a "dangling" keyword or end.
    #
    #   1 def bark
    #   2
    #   3 end
    #
    # If lines 2 and 3 are in the block, then when this method is
    # run it would see it is unbalanced, but that acquiring line 1
    # would make it balanced, so that's what it does.
    def lookahead_balance_one_line
      kw_count = 0
      end_count = 0
      lines.each do |line|
        kw_count += 1 if line.is_kw?
        end_count += 1 if line.is_end?
      end

      return self if kw_count == end_count # nothing to balance

      @scanner.commit_if_changed # Rollback point if we don't find anything to optimize

      # Try to eat up empty lines
      @scanner.scan(
        up: ->(line, _, _) { line.hidden? || line.empty? },
        down: ->(line, _, _) { line.hidden? || line.empty? }
      )

      # More ends than keywords, check if we can balance expanding up
      next_up = @scanner.next_up
      next_down = @scanner.next_down
      case end_count - kw_count
      when 1
        if next_up&.is_kw? && next_up.indent >= @target_indent
          @scanner.scan(
            up: ->(line, _, _) { line == next_up },
            down: ->(line, _, _) { false }
          )
          @scanner.commit_if_changed
        end
      when -1
        if next_down&.is_end? && next_down.indent >= @target_indent
          @scanner.scan(
            up: ->(line, _, _) { false },
            down: ->(line, _, _) { line == next_down }
          )
          @scanner.commit_if_changed
        end
      end
      # Rollback any uncommitted changes
      @scanner.stash_changes

      self
    end

    # Finds code lines at the same or greater indentation and adds them
    # to the block
    def scan_neighbors_not_empty
      @target_indent = @orig_indent
      scan_while { |line| line.not_empty? && line.indent >= @target_indent }
    end

    # Scan blocks based on indentation of next line above/below block
    #
    # Determines indentaion of the next line above/below the current block.
    #
    # Normally this is called when a block has expanded to capture all "neighbors"
    # at the same (or greater) indentation and needs to expand out. For example
    # the `def/end` lines surrounding a method.
    def scan_adjacent_indent
      before_after_indent = []

      before_after_indent << (@scanner.next_up&.indent || 0)
      before_after_indent << (@scanner.next_down&.indent || 0)

      @target_indent = before_after_indent.min
      scan_while { |line| line.not_empty? && line.indent >= @target_indent }

      self
    end

    # Return the currently matched lines as a `CodeBlock`
    #
    # When a `CodeBlock` is created it will gather metadata about
    # itself, so this is not a free conversion. Avoid allocating
    # more CodeBlock's than needed
    def code_block
      CodeBlock.new(lines: lines)
    end

    # Returns the lines matched by the current scan as an
    # array of CodeLines
    def lines
      @scanner.lines
    end

    # Manageable rspec errors
    def inspect
      "#<#{self.class}:0x0000123843lol >"
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # Keeps track of what elements are in the queue in
  # priority and also ensures that when one element
  # engulfs/covers/eats another that the larger element
  # evicts the smaller element
  class PriorityEngulfQueue
    def initialize
      @queue = PriorityQueue.new
    end

    def to_a
      @queue.to_a
    end

    def empty?
      @queue.empty?
    end

    def length
      @queue.length
    end

    def peek
      @queue.peek
    end

    def pop
      @queue.pop
    end

    def push(block)
      prune_engulf(block)
      @queue << block
      flush_deleted

      self
    end

    private def flush_deleted
      while @queue&.peek&.deleted?
        @queue.pop
      end
    end

    private def prune_engulf(block)
      # If we're about to pop off the same block, we can skip deleting
      # things from the frontier this iteration since we'll get it
      # on the next iteration
      return if @queue.peek && (block <=> @queue.peek) == 1

      if block.starts_at != block.ends_at # A block of size 1 cannot engulf another
        @queue.to_a.each { |b|
          if b.starts_at >= block.starts_at && b.ends_at <= block.ends_at
            b.delete
            true
          end
        }
      end
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # Converts a SyntaxError message to a path
  #
  # Handles the case where the filename has a colon in it
  # such as on a windows file system: https://github.com/ruby/syntax_suggest/issues/111
  #
  # Example:
  #
  #    message = "/tmp/scratch:2:in `require_relative': /private/tmp/bad.rb:1: syntax error, unexpected `end' (SyntaxError)"
  #    puts PathnameFromMessage.new(message).call.name
  #    # => "/tmp/scratch.rb"
  #
  class PathnameFromMessage
    EVAL_RE = /^\(eval.*\):\d+/
    STREAMING_RE = /^-:\d+/
    attr_reader :name

    def initialize(message, io: $stderr)
      @line = message.lines.first
      @parts = @line.split(":")
      @guess = []
      @name = nil
      @io = io
    end

    def call
      if skip_missing_file_name?
        if ENV["SYNTAX_SUGGEST_DEBUG"]
          @io.puts "SyntaxSuggest: Could not find filename from #{@line.inspect}"
        end
      else
        until stop?
          @guess << @parts.shift
          @name = Pathname(@guess.join(":"))
        end

        if @parts.empty?
          @io.puts "SyntaxSuggest: Could not find filename from #{@line.inspect}"
          @name = nil
        end
      end

      self
    end

    def stop?
      return true if @parts.empty?
      return false if @guess.empty?

      @name&.exist?
    end

    def skip_missing_file_name?
      @line.match?(EVAL_RE) || @line.match?(STREAMING_RE)
    end
  end
end
# frozen_string_literal: true
# frozen_string_literal: true

module SyntaxSuggest
  module Capture
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  module Capture
    # Shows the context around code provided by "falling" indentation
    #
    # If this is the original code lines:
    #
    #   class OH
    #     def hello
    #       it "foo" do
    #     end
    #   end
    #
    # And this is the line that is captured
    #
    #       it "foo" do
    #
    # It will yield its surrounding context:
    #
    #   class OH
    #     def hello
    #     end
    #   end
    #
    # Example:
    #
    #   FallingIndentLines.new(
    #       block: block,
    #       code_lines: @code_lines
    #   ).call do |line|
    #     @lines_to_output << line
    #   end
    #
    class FallingIndentLines
      def initialize(code_lines:, block:)
        @lines = nil
        @scanner = ScanHistory.new(code_lines: code_lines, block: block)
        @original_indent = block.current_indent
      end

      def call(&yieldable)
        last_indent_up = @original_indent
        last_indent_down = @original_indent

        @scanner.commit_if_changed
        @scanner.scan(
          up: ->(line, _, _) {
            next true if line.empty?

            if line.indent < last_indent_up
              yieldable.call(line)
              last_indent_up = line.indent
            end
            true
          },
          down: ->(line, _, _) {
            next true if line.empty?

            if line.indent < last_indent_down
              yieldable.call(line)
              last_indent_down = line.indent
            end
            true
          }
        )
        @scanner.stash_changes
      end
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  module Capture
    # Shows surrounding kw/end pairs
    #
    # The purpose of showing these extra pairs is due to cases
    # of ambiguity when only one visible line is matched.
    #
    # For example:
    #
    #     1  class Dog
    #     2    def bark
    #     4    def eat
    #     5    end
    #     6  end
    #
    # In this case either line 2 could be missing an `end` or
    # line 4 was an extra line added by mistake (it happens).
    #
    # When we detect the above problem it shows the issue
    # as only being on line 2
    #
    #     2    def bark
    #
    # Showing "neighbor" keyword pairs gives extra context:
    #
    #     2    def bark
    #     4    def eat
    #     5    end
    #
    #
    # Example:
    #
    #   lines = BeforeAfterKeywordEnds.new(
    #     block: block,
    #     code_lines: code_lines
    #   ).call()
    #
    class BeforeAfterKeywordEnds
      def initialize(code_lines:, block:)
        @scanner = ScanHistory.new(code_lines: code_lines, block: block)
        @original_indent = block.current_indent
      end

      def call
        lines = []

        @scanner.scan(
          up: ->(line, kw_count, end_count) {
            next true if line.empty?
            break if line.indent < @original_indent
            next true if line.indent != @original_indent

            # If we're going up and have one complete kw/end pair, stop
            if kw_count != 0 && kw_count == end_count
              lines << line
              break
            end

            lines << line if line.is_kw? || line.is_end?
            true
          },
          down: ->(line, kw_count, end_count) {
            next true if line.empty?
            break if line.indent < @original_indent
            next true if line.indent != @original_indent

            # if we're going down and have one complete kw/end pair,stop
            if kw_count != 0 && kw_count == end_count
              lines << line
              break
            end

            lines << line if line.is_kw? || line.is_end?
            true
          }
        )
        @scanner.stash_changes

        lines
      end
    end
  end
end

module SyntaxSuggest
  # Turns a "invalid block(s)" into useful context
  #
  # There are three main phases in the algorithm:
  #
  # 1. Sanitize/format input source
  # 2. Search for invalid blocks
  # 3. Format invalid blocks into something meaningful
  #
  # This class handles the third part.
  #
  # The algorithm is very good at capturing all of a syntax
  # error in a single block in number 2, however the results
  # can contain ambiguities. Humans are good at pattern matching
  # and filtering and can mentally remove extraneous data, but
  # they can't add extra data that's not present.
  #
  # In the case of known ambiguious cases, this class adds context
  # back to the ambiguity so the programmer has full information.
  #
  # Beyond handling these ambiguities, it also captures surrounding
  # code context information:
  #
  #   puts block.to_s # => "def bark"
  #
  #   context = CaptureCodeContext.new(
  #     blocks: block,
  #     code_lines: code_lines
  #   )
  #
  #   lines = context.call.map(&:original)
  #   puts lines.join
  #   # =>
  #     class Dog
  #       def bark
  #     end
  #
  class CaptureCodeContext
    attr_reader :code_lines

    def initialize(blocks:, code_lines:)
      @blocks = Array(blocks)
      @code_lines = code_lines
      @visible_lines = @blocks.map(&:visible_lines).flatten
      @lines_to_output = @visible_lines.dup
    end

    def call
      @blocks.each do |block|
        capture_first_kw_end_same_indent(block)
        capture_last_end_same_indent(block)
        capture_before_after_kws(block)
        capture_falling_indent(block)
      end

      sorted_lines
    end

    def sorted_lines
      @lines_to_output.select!(&:not_empty?)
      @lines_to_output.uniq!
      @lines_to_output.sort!

      @lines_to_output
    end

    # Shows the context around code provided by "falling" indentation
    #
    # Converts:
    #
    #       it "foo" do
    #
    # into:
    #
    #   class OH
    #     def hello
    #       it "foo" do
    #     end
    #   end
    #
    def capture_falling_indent(block)
      Capture::FallingIndentLines.new(
        block: block,
        code_lines: @code_lines
      ).call do |line|
        @lines_to_output << line
      end
    end

    # Shows surrounding kw/end pairs
    #
    # The purpose of showing these extra pairs is due to cases
    # of ambiguity when only one visible line is matched.
    #
    # For example:
    #
    #     1  class Dog
    #     2    def bark
    #     4    def eat
    #     5    end
    #     6  end
    #
    # In this case either line 2 could be missing an `end` or
    # line 4 was an extra line added by mistake (it happens).
    #
    # When we detect the above problem it shows the issue
    # as only being on line 2
    #
    #     2    def bark
    #
    # Showing "neighbor" keyword pairs gives extra context:
    #
    #     2    def bark
    #     4    def eat
    #     5    end
    #
    def capture_before_after_kws(block)
      return unless block.visible_lines.count == 1

      around_lines = Capture::BeforeAfterKeywordEnds.new(
        code_lines: @code_lines,
        block: block
      ).call

      around_lines -= block.lines

      @lines_to_output.concat(around_lines)
    end

    # When there is an invalid block with a keyword
    # missing an end right before another end,
    # it is unclear where which keyword is missing the
    # end
    #
    # Take this example:
    #
    #   class Dog       # 1
    #     def bark      # 2
    #       puts "woof" # 3
    #   end             # 4
    #
    # However due to https://github.com/ruby/syntax_suggest/issues/32
    # the problem line will be identified as:
    #
    #  > class Dog       # 1
    #
    # Because lines 2, 3, and 4 are technically valid code and are expanded
    # first, deemed valid, and hidden. We need to un-hide the matching end
    # line 4. Also work backwards and if there's a mis-matched keyword, show it
    # too
    def capture_last_end_same_indent(block)
      return if block.visible_lines.length != 1
      return unless block.visible_lines.first.is_kw?

      visible_line = block.visible_lines.first
      lines = @code_lines[visible_line.index..block.lines.last.index]

      # Find first end with same indent
      # (this would return line 4)
      #
      #   end             # 4
      matching_end = lines.detect { |line| line.indent == block.current_indent && line.is_end? }
      return unless matching_end

      @lines_to_output << matching_end

      # Work backwards from the end to
      # see if there are mis-matched
      # keyword/end pairs
      #
      # Return the first mis-matched keyword
      # this would find line 2
      #
      #     def bark      # 2
      #       puts "woof" # 3
      #   end             # 4
      end_count = 0
      kw_count = 0
      kw_line = @code_lines[visible_line.index..matching_end.index].reverse.detect do |line|
        end_count += 1 if line.is_end?
        kw_count += 1 if line.is_kw?

        !kw_count.zero? && kw_count >= end_count
      end
      return unless kw_line
      @lines_to_output << kw_line
    end

    # The logical inverse of `capture_last_end_same_indent`
    #
    # When there is an invalid block with an `end`
    # missing a keyword right after another `end`,
    # it is unclear where which end is missing the
    # keyword.
    #
    # Take this example:
    #
    #   class Dog       # 1
    #       puts "woof" # 2
    #     end           # 3
    #   end             # 4
    #
    # the problem line will be identified as:
    #
    #  > end            # 4
    #
    # This happens because lines 1, 2, and 3 are technically valid code and are expanded
    # first, deemed valid, and hidden. We need to un-hide the matching keyword on
    # line 1. Also work backwards and if there's a mis-matched end, show it
    # too
    def capture_first_kw_end_same_indent(block)
      return if block.visible_lines.length != 1
      return unless block.visible_lines.first.is_end?

      visible_line = block.visible_lines.first
      lines = @code_lines[block.lines.first.index..visible_line.index]
      matching_kw = lines.reverse.detect { |line| line.indent == block.current_indent && line.is_kw? }
      return unless matching_kw

      @lines_to_output << matching_kw

      kw_count = 0
      end_count = 0
      orphan_end = @code_lines[matching_kw.index..visible_line.index].detect do |line|
        kw_count += 1 if line.is_kw?
        end_count += 1 if line.is_end?

        end_count >= kw_count
      end

      return unless orphan_end
      @lines_to_output << orphan_end
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # Outputs code with highlighted lines
  #
  # Whatever is passed to this class will be rendered
  # even if it is "marked invisible" any filtering of
  # output should be done before calling this class.
  #
  #   DisplayCodeWithLineNumbers.new(
  #     lines: lines,
  #     highlight_lines: [lines[2], lines[3]]
  #   ).call
  #   # =>
  #       1
  #       2  def cat
  #     > 3    Dir.chdir
  #     > 4    end
  #       5  end
  #       6
  class DisplayCodeWithLineNumbers
    TERMINAL_HIGHLIGHT = "\e[1;3m" # Bold, italics
    TERMINAL_END = "\e[0m"

    def initialize(lines:, highlight_lines: [], terminal: false)
      @lines = Array(lines).sort
      @terminal = terminal
      @highlight_line_hash = Array(highlight_lines).each_with_object({}) { |line, h| h[line] = true }
      @digit_count = @lines.last&.line_number.to_s.length
    end

    def call
      @lines.map do |line|
        format_line(line)
      end.join
    end

    private def format_line(code_line)
      # Handle trailing slash lines
      code_line.original.lines.map.with_index do |contents, i|
        format(
          empty: code_line.empty?,
          number: (code_line.number + i).to_s,
          contents: contents,
          highlight: @highlight_line_hash[code_line]
        )
      end.join
    end

    private def format(contents:, number:, empty:, highlight: false)
      string = +""
      string << if highlight
        "> "
      else
        "  "
      end

      string << number.rjust(@digit_count).to_s
      if empty
        string << contents
      else
        string << "  "
        string << TERMINAL_HIGHLIGHT if @terminal && highlight
        string << contents
        string << TERMINAL_END if @terminal
      end
      string
    end
  end
end

module SyntaxSuggest
  # Used for formatting invalid blocks
  class DisplayInvalidBlocks
    attr_reader :filename

    def initialize(code_lines:, blocks:, io: $stderr, filename: nil, terminal: DEFAULT_VALUE)
      @io = io
      @blocks = Array(blocks)
      @filename = filename
      @code_lines = code_lines

      @terminal = (terminal == DEFAULT_VALUE) ? io.isatty : terminal
    end

    def document_ok?
      @blocks.none? { |b| !b.hidden? }
    end

    def call
      if document_ok?
        return self
      end

      if filename
        @io.puts("--> #{filename}")
        @io.puts
      end
      @blocks.each do |block|
        display_block(block)
      end

      self
    end

    private def display_block(block)
      # Build explanation
      explain = ExplainSyntax.new(
        code_lines: block.lines
      ).call

      # Enhance code output
      # Also handles several ambiguious cases
      lines = CaptureCodeContext.new(
        blocks: block,
        code_lines: @code_lines
      ).call

      # Build code output
      document = DisplayCodeWithLineNumbers.new(
        lines: lines,
        terminal: @terminal,
        highlight_lines: block.lines
      ).call

      # Output syntax error explanation
      explain.errors.each do |e|
        @io.puts e
      end
      @io.puts

      # Output code
      @io.puts(document)
    end

    private def code_with_context
      lines = CaptureCodeContext.new(
        blocks: @blocks,
        code_lines: @code_lines
      ).call

      DisplayCodeWithLineNumbers.new(
        lines: lines,
        terminal: @terminal,
        highlight_lines: @invalid_lines
      ).call
    end
  end
end
# frozen_string_literal: true

module SyntaxSuggest
  # This class is responsible for generating initial code blocks
  # that will then later be expanded.
  #
  # The biggest concern when guessing code blocks, is accidentally
  # grabbing one that contains only an "end". In this example:
  #
  #   def dog
  #     begonn # misspelled `begin`
  #     puts "bark"
  #     end
  #   end
  #
  # The following lines would be matched (from bottom to top):
  #
  #   1) end
  #
  #   2) puts "bark"
  #      end
  #
  #   3) begonn
  #      puts "bark"
  #      end
  #
  # At this point it has no where else to expand, and it will yield this inner
  # code as a block
  class ParseBlocksFromIndentLine
    attr_reader :code_lines

    def initialize(code_lines:)
      @code_lines = code_lines
    end

    # Builds blocks from bottom up
    def each_neighbor_block(target_line)
      scan = AroundBlockScan.new(code_lines: code_lines, block: CodeBlock.new(lines: target_line))
        .force_add_empty
        .force_add_hidden
        .scan_while { |line| line.indent >= target_line.indent }

      neighbors = scan.code_block.lines

      block = CodeBlock.new(lines: neighbors)
      if neighbors.length <= 2 || block.valid?
        yield block
      else
        until neighbors.empty?
          lines = [neighbors.pop]
          while (block = CodeBlock.new(lines: lines)) && block.invalid? && neighbors.any?
            lines.prepend neighbors.pop
          end

          yield block if block
        end
      end
    end
  end
end
