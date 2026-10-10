# Ripper reports each error through the event MRI's parser uses for it:
# `parse_error`, `assign_error`, `alias_error`, `class_name_error` or
# `compile_error`. RubyVM::AbstractSyntaxTree raises SyntaxError with every
# error's message, each with the line it names quoted beneath it.
require "ripper"

class ErrorEvents < Ripper
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

  %i[assign_error alias_error class_name_error].each do |event|
    define_method(:"on_#{event}") do |message, value|
      @seen << "#{event}: #{message}"
      value
    end
  end
end

SOURCES = [
  "class A; return; end",
  "def f; class A; end; end",
  "def f; module M; end; end",
  "def f; A = 1; end",
  "alias $a $1",
  "def f(a, a); end",
  "foo { |a, a| }",
  "->(a, a) {}",
  "def f(a:, a:); end",
  "case 1; in [x, x]; end",
  "case a; in x | 1; end",
  "case a; in 1 | x; end",
  "[1].each { _1; it }",
  "[1].each { it; _1 }",
  "[1].each { _1; [2].each { _1 } }",
  "_1 = 1",
  "self = 1",
  "$1 = 1",
  "class foo; end",
  "x = return",
  "next next",
  "def foo=() = 1",
  "def f; BEGIN {}; end",
  "x = 1; x = *",
  "\"abc",
  "0o8"
].freeze

SOURCES.each do |source|
  puts(source.inspect)
  events = ErrorEvents.new(source)
  events.parse
  events.seen.each { |line| puts("  #{line}") }
  begin
    RubyVM::AbstractSyntaxTree.parse(source)
  rescue SyntaxError => error
    error.message.each_line { |line| puts("  | #{line.chomp}") }
  end
end
