def first_large(values)
  index = -1
  begin
    index += 1
    break if values[index] > 10
  end while index < values.size - 1
  values[index]
end
p(first_large([3, 14, 5]))

def skip_small(values)
  index = 0
  total = 0
  begin
    index += 1
    next if values[index - 1] < 5
    total += values[index - 1]
  end until index >= values.size
  total
end
p(skip_small([3, 14, 5]))

class Sorter
  INVOICE = /\AINV-(\d+)\z/
  SMALL = (1..9)
  LIMIT = 100

  def self.kind(value)
    case value
    when INVOICE then "invoice #{$1}"
    when SMALL then "small"
    when LIMIT then "limit"
    else "other"
    end
  end
end
p(Sorter.kind("INV-1042"))
p(Sorter.kind(4))
p(Sorter.kind(100.0))
p(Sorter.kind(:pending))

p(case 4.0 when 4 then :whole end)
p(case 4 when 4.0 then :float end)
p(case 4.5 when 4 then :whole else :fraction end)
