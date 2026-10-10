require "mutex_m"
ledger = []
ledger.extend Mutex_m
ledger.mu_synchronize { ledger << :posted }
p [ledger, ledger.mu_locked?]
ledger.mu_lock
p ledger.mu_locked?
ledger.mu_unlock

require "nkf"
sjis = "\x93\xFA\x96\x7B".force_encoding("Shift_JIS")
p NKF.guess(sjis)
p NKF.nkf("-w", sjis)
p NKF.nkf("-s", "日本").bytes
require "kconv"
p sjis.toutf8
p NKF::VERSION

require "racc/parser"
require "racc/grammarfileparser"
require "racc/parserfilegenerator"
grammar = <<~GRAMMAR
  class Calculator
  rule
    expression: expression '+' NUMBER { result = val[0] + val[2] }
              | NUMBER
  end
GRAMMAR
parsed = Racc::GrammarFileParser.parse(grammar)
states = parsed.grammar.states
generator = Racc::ParserFileGenerator.new(states, parsed.params.dup)
source = generator.generate_parser
eval source
Calculator.class_eval do
  def parse(tokens)
    @tokens = tokens.dup
    do_parse
  end

  def next_token
    @tokens.shift || [false, "$"]
  end
end
p Calculator.new.parse([[:NUMBER, 2], ["+", "+"], [:NUMBER, 3], ["+", "+"], [:NUMBER, 4]])
p Racc::Parser::Racc_Runtime_Core_Version

require "resolv-replace"
p IPSocket.getaddress("127.0.0.1")

require "rinda/tuplespace"
space = Rinda::TupleSpace.new
space.write [:invoice, 1042, 99]
space.write [:invoice, 1043, 15]
p space.read_all([:invoice, nil, nil]).size
p space.take([:invoice, 1043, nil])
p space.read_all([:invoice, nil, nil])

require "rinda/ring"
p Rinda::Ring_PORT
p NKF.nkf("-e -m0", "日本").bytes
p "日本".tosjis.encoding
