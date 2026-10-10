# A method written at the top level runs with `main` as `self`, however it
# is called, and a call written with parentheses reaches the method even
# where a local of the same name exists.
def whoami
  self
end

p whoami
p whoami()
p method(:whoami).call
p method(:whoami).receiver

class Caller
  def ask = whoami
end
p Caller.new.ask.class

def label = :method
label = "local"
p label
p label()

count = 3
begin
  count()
rescue NoMethodError => error
  puts error.message
end

format = 5
p format("%03d", 7)
p format
