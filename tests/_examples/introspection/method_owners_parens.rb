# A Method names the class that holds it, whichever singleton class the
# lookup walked through to reach it. Materializing a singleton class
# somewhere else leaves that answer alone.
p(String.method(:include).owner)
p(String.method(:include).receiver)
p(String.method(:include).name)

Array.singleton_class
p(String.method(:include).owner)

# A class method made private is hidden from the outside and left out of the
# names the class reports, while the owner of an unrelated method stands.
class Ledger
  def self.open_entry; :open; end
  def self.closed_entry; :closed; end
  private_class_method(:closed_entry)
end

p(Ledger.open_entry)
p(Ledger.respond_to?(:closed_entry))
p(Ledger.singleton_methods(false).sort)
begin
  Ledger.closed_entry
rescue NoMethodError => problem
  p(problem.class)
end
p(Ledger.singleton_class.private_instance_methods(false).include?(:closed_entry))
p(String.method(:include).owner)

# One made public again answers from the outside.
Ledger.public_class_method(:closed_entry)
p(Ledger.closed_entry)
p(Ledger.singleton_methods(false).sort)
