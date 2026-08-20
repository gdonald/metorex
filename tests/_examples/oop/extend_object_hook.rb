module Greeting
  GREETING = :hello

  def greet
    "hello test"
  end
end

module RecordingExtend
  class << self
    def extend_object obj
      $recorded << :private_hook
    end
    private :extend_object
  end
end

module PublicExtend
  def self.extend_object obj
    $recorded << :public_hook
  end
end

$recorded = []

plain = Object.new
plain.extend Greeting
puts plain.greet
puts plain.singleton_class.const_get(:GREETING).inspect

Object.new.extend RecordingExtend
Object.new.extend PublicExtend
puts $recorded.inspect

skipped = Object.new
skipped.extend RecordingExtend
puts skipped.respond_to?(:greet).inspect

frozen = Object.new.freeze
begin
  Greeting.send :extend_object, frozen
rescue RuntimeError => error
  puts error.class
end
puts frozen.is_a?(Greeting).inspect
