# Each form that binds a local makes `name [1]` index that local from then
# on, where before it the same words call a method named `name` with an
# Array.
class IndexedError < StandardError
  def [] index
    message[index]
  end
end

def first *arguments
  [:method, arguments]
end
def second *arguments
  [:method, arguments]
end
def rest *arguments
  [:method, arguments]
end
def error *arguments
  [:method, arguments]
end
def item *arguments
  [:method, arguments]
end
def hidden *arguments
  [:method, arguments]
end
def match *arguments
  [:method, arguments]
end
def held *arguments
  [:method, arguments]
end
def matched *arguments
  [:method, arguments]
end

p first [1]
(first, (second, *rest)), _ = [["a", ["b", "c", "d"]], nil]
p first [0], second [0], rest [1]

begin
  raise IndexedError, "boom"
rescue => error
  p error [0]
end

for item in [[7, 8]]
  p item [1]
end

[[5, 6]].each { |pair; hidden| hidden = pair; p hidden [0] }
p hidden [0]

[[3, 4]].each { p _1 [1] }
[[1, 2]].each do p _1 [0] end

if /(?<match>\w+)/ =~ "word"
  p match [0]
end

held = [9]
case [[9]]
in [^held] then p held [0]
end

case [1, 2]
in [Integer, _] | [String, _] => matched then p matched
end
p matched [0]
