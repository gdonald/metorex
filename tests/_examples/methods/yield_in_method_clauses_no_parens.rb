# A method's own rescue, else and ensure clauses belong to its body, so
# `yield` there hands values to the block the method was called with.
def recovering
  raise ArgumentError, "first try"
rescue ArgumentError => error
  yield error.message
end

def finishing
  :body
ensure
  yield :ensure
end

def succeeding
  :body
rescue
  :never
else
  yield :else
end

p recovering { |message| "recovered from #{message}" }
p finishing { |where| puts "ran in #{where}" }
p succeeding { |where| "answered from #{where}" }
