class << $receiver = Object.new
  def greet
    :hello
  end

  def shout
    :HELLO
  end
end

puts $receiver.greet.inspect
puts $receiver.shout.inspect
puts $receiver.methods(false).sort.inspect

class Holder
  def build
    class << @target = Object.new
      def describe
        :built
      end
    end
    @target
  end
end

holder = Holder.new
built = holder.build
puts built.describe.inspect

plain = Object.new
class << plain
  def still_works
    :yes
  end
end
puts plain.still_works.inspect

names = %i[alpha beta gamma]
puts names.inspect
puts names.first.class.name
words = %w[alpha beta]
puts words.first.class.name
parens = %i(one two)
puts parens.inspect
