class Settings
  if false
    path = "/etc/app.conf"
  end
  DEFAULT_PATH = path || "/usr/local/etc/app.conf"
end
p(Settings::DEFAULT_PATH)

module Labels
  def label!(text)
    @label = text
  end

  def label
    @label
  end
end

orders = [1042, 1043]
orders.extend(Labels)
orders.label!("open orders")
p(orders.label)
p(orders.instance_variables)

class Hash
  def stamp(value) = (@stamp = value)
  def stamped = @stamp
end
ledger = {}
ledger.stamp("2024-06-01")
p(ledger.stamped)
p([].freeze.then { |frozen| frozen.instance_variable_set(:@x, 1) rescue $!.class })

pairs = [[:a, 1], [:b, 2]].map do |key, value|
  it = value * 10
  [key, it]
end
p(pairs)
