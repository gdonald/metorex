def report
  yield
rescue Exception => error
  puts(error.detailed_message(highlight: false).gsub(__FILE__, "FILE"))
  puts("--")
end

def shipping_cost(weight, zone) = weight * zone

class Parcel
  attr_reader :weight
end

total = 10
report { total = totl + 1 }
report { order = nil; order.length }
report { sum = 1 + "2" }
report { shipping_cost(4) }
report { rows = nil; rows[0] }
report { nil.label = "fragile" }
report { [1].map(&:nope) }
report { Parcel.new.weight.round(nil) }
report { :upcase.to_proc.call }
report { Comparable.instance_method(:clamp).bind_call(1, "x", 2) }

begin
  Parcel.new.volume
rescue NoMethodError => error
  spot = ErrorHighlight.spot(error)
  p(spot.values_at(:first_lineno, :first_column, :last_lineno, :last_column))
  p(spot[:snippet])
end
