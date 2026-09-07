# A Data class holds a fixed set of members and cannot be changed once made.
Measure = Data.define :amount, :unit

p Measure.members
p Measure.new(42, "km").amount
p Measure.new(amount: 42, unit: "km").unit
p Measure[3, "mi"].to_h
p Measure.new(1, "m").frozen?
p Measure.new(1, "m").inspect

first = Measure.new 42, "km"
second = Measure.new 42, "km"
p first == second
p first.eql? second
p first.hash == second.hash
p first == Measure.new(42, "mi")

p first.with.equal? first
p first.with(unit: "m").to_h
p first.deconstruct
p first.deconstruct_keys [:amount]
p first.deconstruct_keys nil
p first.to_h { |name, value| [name.to_s, value] }

Named = Data.define(:title, :year) do
  def label
    "#{title} (#{year})"
  end
end
p Named.new("Rain", 1999).label

begin
  Measure.new amount: 1
rescue ArgumentError => error
  p error.message
end

begin
  Measure.new amount: 1, unit: "m", system: "metric"
rescue ArgumentError => error
  p error.message
end
