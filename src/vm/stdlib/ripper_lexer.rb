# Ripper::Lexer collects the scanner events as positioned tokens, and
# Ripper.lex and Ripper.tokenize return them in source order.

require 'ripper/core'

class Ripper
  def self.tokenize(src, filename = "-", lineno = 1, **options)
    Lexer.new(src, filename, lineno).tokenize(**options)
  end

  def self.lex(src, filename = "-", lineno = 1, **options)
    Lexer.new(src, filename, lineno).lex(**options)
  end

  class Lexer < ::Ripper
    # A lexer state, shown by the names of its bits.
    class State
      attr_reader :to_int, :to_s

      def initialize(value)
        @to_int = value
        @to_s = Ripper.lex_state_name(value)
        freeze
      end

      def [](index)
        case index
        when 0, :to_int then @to_int
        when 1, :to_s then @event
        end
      end

      alias to_i to_int
      alias inspect to_s

      def pretty_print(printer) = printer.text(to_s)

      def ==(other) = super || to_int == other

      def &(other) = self.class.new(to_int & other)

      def |(other) = self.class.new(to_int | other)

      def allbits?(bits) = to_int.allbits?(bits)

      def anybits?(bits) = to_int.anybits?(bits)

      def nobits?(bits) = to_int.nobits?(bits)
    end

    # One token: where it starts, its event, its text and the lexer state.
    class Elem
      attr_accessor :pos, :event, :tok, :state, :message

      def initialize(pos, event, tok, state, message = nil)
        @pos = pos
        @event = event
        @tok = tok
        @state = State.new(state)
        @message = message
      end

      def [](index)
        case index
        when 0, :pos then @pos
        when 1, :event then @event
        when 2, :tok then @tok
        when 3, :state then @state
        when 4, :message then @message
        end
      end

      def inspect
        "#<#{self.class}: #{event}@#{pos[0]}:#{pos[1]}:#{state}: #{tok.inspect}#{': ' if message}#{message}>"
      end

      alias to_s inspect

      def pretty_print(printer)
        printer.group(2, "#<#{self.class}:", ">") do
          printer.breakable
          printer.text("#{event}@#{pos[0]}:#{pos[1]}")
          printer.breakable
          state.pretty_print(printer)
          printer.breakable
          printer.text("token: ")
          tok.pretty_print(printer)
          if message
            printer.breakable
            printer.text("message: ")
            printer.text(message)
          end
        end
      end

      def to_a
        message ? [@pos, @event, @tok, @state, @message] : [@pos, @event, @tok, @state]
      end
    end

    attr_reader :errors

    def tokenize(**options)
      parse(**options).sort_by(&:pos).map(&:tok)
    end

    def lex(**options)
      parse(**options).sort_by(&:pos).map(&:to_a)
    end

    # The tokens and the errors together, each error placed on the token
    # it was found at.
    def scan(**options)
      result = (parse(**options) + errors + @stack.flatten).uniq.sort_by { |elem| [*elem.pos, elem.message ? -1 : 0] }
      result.each_with_index do |elem, index|
        previous = result[index - 1]
        next unless elem.event == :on_parse_error && elem.tok.empty? && previous &&
                    previous.pos[0] == elem.pos[0] && previous.pos[1] + previous.tok.size == elem.pos[1]

        elem.tok = previous.tok
        elem.pos[1] = previous.pos[1]
        result[index - 1] = elem
        result[index] = previous
      end
      result
    end

    def parse(raise_errors: false)
      @errors = []
      @buf = []
      @stack = []
      super()
      @buf = @stack.pop until @stack.empty?
      raise SyntaxError, @errors.map(&:message).join(" ;") if raise_errors && !@errors.empty?

      @buf.flatten!
      result = @buf
      unless result.empty?
        loop do
          @buf = []
          super()
          @buf.flatten!
          break if @buf.empty?

          result.concat(@buf)
        end
      end
      result
    end

    private

    def on_heredoc_dedent(value, width)
      ignored = []
      heredoc = @buf.last
      if heredoc.is_a?(Array)
        heredoc.each_with_index do |elem, index|
          next unless elem.is_a?(Elem) && elem.event == :on_tstring_content && elem.pos[1].zero?

          original = elem.tok.dup if width.positive? && elem.tok.match?(/\A\s/)
          removed = dedent_string(elem.tok, width)
          next unless removed.positive?

          if elem.tok.empty?
            elem.tok = original[0, removed]
            elem.event = :on_ignored_sp
            next
          end
          ignored << [index, Elem.new(elem.pos.dup, :on_ignored_sp, original[0, removed], elem.state)]
          elem.pos[1] += removed
        end
      end
      ignored.reverse_each { |index, elem| heredoc[index, 0] = [elem] }
      value
    end

    def on_heredoc_beg(token)
      @stack.push(@buf)
      nested = []
      @buf.push(nested)
      @buf = nested
      @buf.push(Elem.new([lineno, column], __callee__, token, state))
    end

    def on_heredoc_end(token)
      @buf.push(Elem.new([lineno, column], __callee__, token, state))
      @buf = @stack.pop unless @stack.empty?
    end

    def _push_token(token)
      elem = Elem.new([lineno, column], __callee__, token, state)
      @buf.push(elem)
      elem
    end

    def on_error1(message)
      @errors.push(Elem.new([lineno, column], __callee__, token, state, message))
    end

    def on_error2(message, elem)
      elem = if elem
               Elem.new(elem.pos, __callee__, elem.tok, elem.state, message)
             else
               Elem.new([lineno, column], __callee__, token, state, message)
             end
      @errors.push(elem)
    end

    PARSER_EVENTS.grep(/_error\z/) do |event|
      alias_method "on_#{event}", "on_error#{PARSER_EVENT_TABLE.fetch(event)}"
    end

    alias compile_error on_error1

    (SCANNER_EVENTS.map { |event| :"on_#{event}" } - private_instance_methods(false)).each do |event|
      alias_method event, :_push_token
    end
  end

  def self.slice(src, pattern, n = 0)
    match = token_match(src, pattern)
    match ? match.string(n) : nil
  end

  def self.token_match(src, pattern)
    TokenPattern.compile(pattern).match(src)
  end

  # A pattern over token kinds, written with token names and regular
  # expression operators: `'ident \( ident \)'`.
  class TokenPattern
    class Error < ::StandardError; end
    class CompileError < Error; end
    class MatchError < Error; end

    class << self
      alias compile new
    end

    def initialize(pattern)
      @source = pattern
      @re = compile(pattern)
    end

    def match(str)
      match_list(::Ripper.lex(str))
    end

    def match_list(tokens)
      found = @re.match(map_tokens(tokens))
      found ? MatchData.new(tokens, found) : nil
    end

    private

    def compile(pattern)
      if (invalid = /[^\w\s$()\[\]{}?*+.]/.match(pattern))
        raise CompileError, "invalid char in pattern: #{invalid[0].inspect}"
      end

      buffer = +""
      pattern.scan(/(?:\w+|\$\(|[()\[\]{}?*+.]+)/) do |piece|
        case piece
        when /\w/ then buffer.concat(map_token(piece))
        when "$(" then buffer.concat("(")
        when "(" then buffer.concat("(?:")
        when /[?*\[\]).]/ then buffer.concat(piece)
        else raise "must not happen"
        end
      end
      Regexp.compile(buffer)
    rescue RegexpError => e
      raise CompileError, e.message
    end

    def map_tokens(tokens)
      tokens.map { |_pos, type, _str| map_token(type.to_s.delete_prefix("on_")) }.join
    end

    MAP = {}
    seed = ("a".."z").to_a + ("A".."Z").to_a + ("0".."9").to_a
    SCANNER_EVENT_TABLE.each_key do |event|
      raise CompileError, "[RIPPER FATAL] too many system token" if seed.empty?

      MAP[event.to_s.delete_prefix("on_")] = seed.shift
    end

    def map_token(token)
      MAP[token] or raise CompileError, "unknown token: #{token}"
    end

    class MatchData
      def initialize(tokens, match)
        @tokens = tokens
        @match = match
      end

      def string(n = 0)
        return nil unless @match

        match(n).join
      end

      private

      def match(n = 0)
        return [] unless @match

        @tokens[@match.begin(n)...@match.end(n)].map { |_pos, _type, str| str }
      end
    end
  end
end
