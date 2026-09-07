class Key
  def initialize(name)
    @name = name
  end

  def hash
    7
  end

  def eql?(other)
    other.is_a?(Key) && other.name == @name
  end

  def name
    @name
  end
end

matched = {}
matched[Key.new("a")] = 1
p matched.has_key? Key.new("a")
p matched.has_key? Key.new("b")
p matched.fetch Key.new("a")

begin
  matched.fetch Key.new("z")
rescue KeyError => error
  puts error.message.start_with?("key not found:")
  p error.receiver.size
  p error.key.class.to_s
end
p matched.fetch Key.new("z"), :fallback
p matched.fetch(Key.new("z")) { :from_block }

base = { a: 1, b: 2 }
p base.merge({ b: 20, c: 3 })
p base.merge({ b: 20 }, { d: 4 })
p base.merge({ b: 20 }) { |key, ours, theirs| ours + theirs }
p base
copy = base.dup
p copy.merge! c: 3
p copy.update d: 4
p copy.replace({ z: 26 })

p base.select! { |name, value| value > 1 }
p base
p({ a: 1 }.reject! { |name, value| false })
p({ a: 1, b: nil }.compact!)
p({ a: 1 }.compact!)

marked = { a: 1 }.compare_by_identity
p marked.compare_by_identity?
p marked.slice(:a).compare_by_identity?
p marked.invert.compare_by_identity?

defaults = Hash.new
defaults.default = 9
p defaults[:missing]
p defaults.default
computed = Hash.new { |hash, key| key.to_s }
p computed[:name]
p computed.default :other
p computed.default_proc.class.to_s

pairs = []
{ x: 1, y: 2 }.each { |pair| pairs << pair }
p pairs
p({ x: 1, y: 2 }.each.size)
p({ x: 1 }.to_h { |name, value| [name.to_s, value + 1] })

p({ nil: 1, false: 2 }.keys)
first = 1
second = 2
p({ first:, second: })

class Counts < Hash
end

tally = Counts.new
tally[:hits] = 3
p tally.class.to_s
p tally.size
p tally.to_hash.class.to_s
p tally[:hits]
