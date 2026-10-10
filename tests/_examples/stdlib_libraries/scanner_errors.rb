# The scanner reports an escape or a number MRI's lexer refuses and reads
# on past it, as MRI's lexer does, so a later syntax error is reported too.
# Each error comes through the Ripper event MRI uses, and
# RubyVM::AbstractSyntaxTree.parse quotes the place each one names.
require "ripper"

class ScannerErrors < Ripper
  attr_reader :seen

  def initialize(*)
    super
    @seen = []
  end

  def on_parse_error(message)
    @seen << "parse_error: #{message}"
  end

  def compile_error(message)
    @seen << "compile_error: #{message}"
  end
end

SOURCES = [
  "\"\\xg\"",
  "\"\\u{zz}\"",
  "\"\\u{61\"",
  "\"\\u12\"",
  "x = 0x + 1",
  "0xg",
  "0b2",
  "@1",
  "@@1a",
  "$-",
  "?\\xg",
  "%(\\xg)",
  "/\\xg/",
  "'\\xg'"
].freeze

SOURCES.each do |source|
  puts(source.inspect)
  errors = ScannerErrors.new(source)
  errors.parse
  errors.seen.each { |line| puts("  #{line}") }
  begin
    RubyVM::AbstractSyntaxTree.parse(source)
  rescue SyntaxError => error
    error.message.each_line { |line| puts("  | #{line.chomp}") }
  end
end

p(Ripper.lex("\"a\\u{zz}b\"").map { |(_, event, text)| [event, text] })
p(Ripper.lex("@1").map { |(_, event, text)| [event, text] })
