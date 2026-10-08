# Ruby appends what a misspelled name may have meant to the report of a
# NameError, NoMethodError, KeyError, LoadError or NoMatchingPatternKeyError.
require("rbconfig")
require("tmpdir")

def suggested
  yield
rescue NameError, KeyError, LoadError, NoMatchingPatternKeyError => error
  p([error.class, error.corrections])
  p(error.detailed_message(highlight: false, error_highlight: false))
end

class Shelf
  def capacity = 12
  def label = "pantry"
end

first_name = "Ann"
suggested { firts_name }
suggested { [1].each { firts_name } }
suggested { Strng }
suggested { Shelf.new.capcity }
suggested { Shelf.new.lable }
suggested { {name: 1}.fetch(:nme) }
suggested { {"alpha" => 1}.fetch("alpah") }
suggested { {name: 1}.fetch("name") }
suggested { require("fileutil") }
suggested { case {name: 1}; in {nmae:}; end }
suggested { Shelf.new.nothing_close }

error = (Shelf.new.capcity rescue $!)
p(error.detailed_message(highlight: false, error_highlight: false, did_you_mean: false))
p(error.detailed_message(highlight: true, error_highlight: false))
p(error.message)

checker = DidYouMean::SpellChecker.new(dictionary: %w[apple banana cherry])
p(checker.correct("appel"))
p(checker.correct("bananna"))
p(checker.correct("grape"))
paths = DidYouMean::TreeSpellChecker.new(dictionary: %w[net/http net/ftp open3])
p(paths.correct("net/htp"))
p(DidYouMean::Formatter.message_for(["first", "second"]))
p(DidYouMean::Formatter.message_for([]))
p(DidYouMean::JaroWinkler.distance("martha", "marhta").round(4))
p(DidYouMean::Levenshtein.distance("kitten", "sitting"))

Dir.mktmpdir do |directory|
  script = File.join(directory, "missing_key.rb")
  File.write(script, "raise KeyError.new(\"missing\", receiver: {name: 1}, key: :nme)\n")
  report = IO.popen([RbConfig.ruby, "missing_key.rb", err: [:child, :out], chdir: directory], &:read)
  puts(report.sub(/\A.*in '<main>': /, ""))
  quiet = IO.popen([RbConfig.ruby, "--disable=did_you_mean", "missing_key.rb", err: [:child, :out], chdir: directory], &:read)
  puts(quiet.sub(/\A.*in '<main>': /, ""))
end
