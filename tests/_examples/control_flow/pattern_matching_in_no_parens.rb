class Point
  attr_reader :x, :y

  def initialize x, y
    @x = x
    @y = y
  end

  def deconstruct
    [@x, @y]
  end

  def deconstruct_keys keys
    { x: @x, y: @y }
  end
end

def describe value
  case value
  in Point(x: 0, y: 0)
    "origin"
  in Point[x, y]
    "point #{x},#{y}"
  in []
    "empty"
  in [Integer => only]
    "one integer #{only}"
  in [_, _, *rest] if rest.empty?
    "a pair"
  in [*, 7, *post]
    "seven, then #{post.length} more"
  in { name: String => name, age: Integer => age }
    "#{name} is #{age}"
  in { role: :admin | :owner }
    "privileged"
  in { id:, **rest }
    "id #{id} with #{rest.keys.length} extra"
  in String | Symbol
    "a name"
  else
    "unknown"
  end
end

puts describe []
puts describe [4]
puts describe [1, 2]
puts describe [1, 7, 8, 9]
puts describe name: "Ada", age: 36
puts describe role: :owner
puts describe id: 11, a: 1, b: 2
puts describe Point.new 0, 0
puts describe Point.new 3, 4
puts describe :ada
puts describe 2.5

wanted = 5
case 5
in ^wanted
  puts "pinned match"
end

config = { port: 8080, host: "localhost" }
config => { port: Integer => port }
puts port

an_integer = (5 in Integer)
a_string = ("5" in Integer)
puts an_integer
puts a_string

case [1, [2, 3]]
in [a, [b, c]]
  puts a + b + c
end

case { status: "ok", data: [1, 2] }
in { status: "ok", data: [first, *] }
  puts "ok starting at #{first}"
end

begin
  case { x: 1 }
  in { y: Integer }
    "matched"
  end
rescue NoMatchingPatternKeyError => missed
  puts missed.message
  puts missed.key
  puts missed.matchee
end

begin
  case [1, 2]
  in [1, 3]
    "matched"
  end
rescue NoMatchingPatternError => missed
  puts missed.message
end

begin
  case { x: 1 }
  in { y: 1 }
    "first"
  in { z: 1 }
    "second"
  end
rescue NoMatchingPatternError => missed
  puts missed.message
end
