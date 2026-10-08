# IRB::Color colors Ruby source for a terminal: keywords, constants, numbers,
# strings, symbols, comments and the names of methods defined and called.
require "irb/color"
require "rbconfig"
require "tmpdir"

source = <<~'CODE'
  # A comment
  require "set"
  module Shipping
    class Crate < Struct.new(:width, :height)
      RATE = 1.5
      def volume(depth = 2) = width * height * depth
      def label
        "#{width}x#{height} #@name $0 #$1"
      end
      alias size volume
      undef label
    end
  end
  crate = Shipping::Crate.new 3, 4r
  p crate.volume, :sym, :"quoted #{1}", %i[a b], %w[c d], {key: 1, "str": 2}
  p nil, true, false, self, __FILE__, __LINE__, ?a, 2i, 0x1f, 1e3, $stdout, $~
  puts `echo hi` if defined?(crate) && !false or not true
  x = /ab+c/i =~ "abbc"
  value = [1, 2].map { it ** 2 }.sum rescue 0
  first, crate.width = 1, 2
  case value when 5 then p 1 else p 2 end
  y = <<~TEXT
    heredoc #{value}
  TEXT
  =begin
  doc
  =end
  __END__
  trailing
CODE

IRB::Color.colorize_code(source, colorable: true).each_line { |line| p line }
p IRB::Color.colorize_code(source, colorable: false).equal?(source)
p IRB::Color.colorize_code("width + depth\n", colorable: true, local_variables: [:width])
p IRB::Color.colorize_code("def (\n\tx\x01", colorable: true, ignore_error: true)
p IRB::Color.colorize_code("\uFEFF# marked\n", colorable: true)
p IRB::Color.colorize("text", [:RED, :BOLD], colorable: true)
p IRB::Color.colorize("text", [:RED], colorable: false)
p IRB::Color.clear(colorable: true), IRB::Color.clear(colorable: false)
p IRB::Color.inspect_colorable?([1, "a", {b: nil}, 1..2, String])
p IRB::Color.inspect_colorable?(Object.new), IRB::Color.inspect_colorable?(Class.new)

Dir.mktmpdir do |directory|
  path = File.join directory, "sample.rb"
  File.write path, "p :piped\n"
  print IO.popen([RbConfig.ruby, "-run", "-e", "colorize", "--", path], &:read)
end
