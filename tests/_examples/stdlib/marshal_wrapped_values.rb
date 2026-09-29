# Marshal writes a String, Regexp, Array or Hash with what it carries: its
# encoding and instance variables, the modules it was extended with, and its
# class when that is a subclass.
module Tagged; end
class Words < Array; end
class Table < Hash; end

p(Marshal.dump("text"))
p(Marshal.dump("text".b))
p(Marshal.dump(:"→"))
p(Marshal.dump(/a./i))
p(Marshal.dump([].extend(Tagged)))
p(Marshal.dump(Words.new))
p(Marshal.dump(Hash.new(0)))
p(Marshal.dump({}.compare_by_identity))
p(Marshal.dump(Table.new))

noted = "note"
noted.instance_variable_set(:@by, "me")
p(Marshal.dump(noted))

begin
  Marshal.dump(Hash.new { |hash, key| key })
rescue TypeError => error
  p(error.message)
end
