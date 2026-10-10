require "stringio"

p(defined?(SyntaxSuggest))

["missing_end.rb", "extra_end.rb"].each do |name|
  path = File.join(__dir__, name)
  io = StringIO.new
  require("syntax_suggest/api")
  SyntaxSuggest.call(io: io, source: File.read(path), filename: name, terminal: false, timeout: 60)
  puts(io.string)
  begin
    load(path)
  rescue SyntaxError => error
    annotation = io.string.sub(name, path)
    p(error.detailed_message(highlight: false).start_with?(annotation))
    p(error.detailed_message(highlight: false, syntax_suggest: false).include?("-->"))
  end
end

p(SyntaxSuggest.valid?("def total\nend\n"))
p(SyntaxSuggest.invalid?(["def total\n", "  def tax\n", "end\n"]))
