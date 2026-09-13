def written_before_the_assignment
  block = proc { counted = 99; counted }
  counted = 1
  [block.call, counted].inspect
end
puts(written_before_the_assignment)

def written_after_the_assignment
  counted = 1
  block = proc { counted = 99; counted }
  [block.call, counted].inspect
end
puts(written_after_the_assignment)

collected = Enumerator.new do |yielder|
  answered = yielder.yield(3)
  yielder << answered << 2 << 1
end
answered = []
collected.each { |value| answered.push(value); value * 2 }
puts(answered.inspect)

def reads_a_later_local_from_a_rescue
  begin
    raise("stopped")
  rescue RuntimeError
    return held.inspect
  end
  held = 1
end
puts(reads_a_later_local_from_a_rescue)
