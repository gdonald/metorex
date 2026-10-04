# A class can take `new` away from itself in its singleton class, after which
# calling it is a NoMethodError and it is no longer listed. A private class
# method names the class or module in the message about calling it.

class Fixed
  class << self
    undef_method :new
    def build = :built
  end
end

begin
  Fixed.new
rescue NoMethodError => trouble
  p trouble.message
end
p Fixed.respond_to?(:new)
p Fixed.singleton_methods
p Fixed.build

class Hidden
  private_class_method :new
end

module Helpers
  def self.assist = :assisted
  private_class_method :assist
end

[-> { Hidden.new }, -> { Helpers.assist }].each do |call|
  call.call
rescue NoMethodError => trouble
  p trouble.message
end
