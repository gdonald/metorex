# Ripper::Filter walks the tokens of a source in order, calling an `on_`
# method for each and threading a value through them.

require 'ripper/lexer'

class Ripper
  class Filter
    def initialize(src, filename = "-", lineno = 1)
      @__lexer = Lexer.new(src, filename, lineno)
      @__line = nil
      @__col = nil
      @__state = nil
    end

    def filename = @__lexer.filename

    def lineno = @__line

    def column = @__col

    def state = @__state

    def parse(init = nil)
      data = init
      @__lexer.lex.each do |pos, event, token, state|
        @__line, @__col = *pos
        @__state = state
        data = if respond_to?(event, true)
                 __send__(event, token, data)
               else
                 on_default(event, token, data)
               end
      end
      data
    end

    private

    def on_default(_event, _token, data) = data
  end
end
