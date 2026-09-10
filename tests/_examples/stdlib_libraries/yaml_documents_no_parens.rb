require 'yaml'

# A scalar, a sequence, and a mapping each read back as what they name.
p YAML.load("--- str")
p YAML.load("--- :locked")
p YAML.load("47")
p YAML.load("--- \n- a\n- b\n- c\n")
p YAML.load("--- [a, b, c]")
p YAML.load("a: b\nc: 2\n")
p YAML.load("- - - one\n    - two\n    - three")
p YAML.load(":user name: This is the user name.")
p YAML.load("---\n")

# A stream holds one document per `---` line.
stream = "---\n- Mark McGwire\n- Sammy Sosa\n---\n- Chicago Cubs\n"
p YAML.load_stream(stream)

documents = []
YAML.load_stream(stream) { |document| documents << document }
p documents.length

# Writing puts the `---` in front and names what has no plain form.
p YAML.dump(:locked)
p YAML.dump("str")
p YAML.dump({ "a" => "b" })
p YAML.dump(["a", "b", "c"])
p YAML.dump_stream("foo", 20, [], {})
p [{ "a" => "b" }, { "b" => "c" }].to_yaml
p Enumerable.to_yaml
p StandardError.new("foobar").to_yaml
p Range.new(1, 3).to_yaml
p (0.0 / 0.0).to_yaml

# A key may be a collection of its own, written under a `?`.
p YAML.load("? # a comment\n  - Detroit Tigers\n  - Chicago Cubs\n:\n  - first\n")

begin
  YAML.load("key1: value\ninvalid_key")
rescue Psych::SyntaxError => problem
  p(problem.class)
end
