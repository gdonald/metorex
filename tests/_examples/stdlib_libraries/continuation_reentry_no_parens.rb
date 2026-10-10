# A continuation called after its callcc block has returned goes back to
# the place callcc was called from, which then answers what the
# continuation was called with.
require "stringio"
$stderr = StringIO.new
require "continuation"
$stderr = STDERR

saved = nil
count = callcc { |continuation| saved = continuation; 0 }
puts count
saved.call count + 1 if count < 2

def counted
  again = nil
  number = callcc { |continuation| again = continuation; 10 }
  puts "in method #{number}"
  again.call number + 1 if number < 12
  :done
end
p counted
