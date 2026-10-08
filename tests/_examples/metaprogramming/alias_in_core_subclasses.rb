# A class below a core class gives a second name to a method the core class
# answers natively, with alias or alias_method.
class Keys < Hash
  alias_method(:each_name, :each_key)
end

Keys[{ a: 1, b: 2 }].each_name { |name| p(name) }

class List < Array
  alias count_of size
  alias pick fetch
  alias every each
end

list = List.new([5, 6])
p(list.count_of, list.pick(1), list.pick(9, :none))
list.every { |item| p(item) }

class Text < String
  alias shout upcase
end

p(Text.new("hi").shout)
