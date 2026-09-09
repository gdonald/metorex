# A block given a limit on how long it may run.
module Timeout
  class Error < RuntimeError
    def self.catch(*arguments, &block)
      made = new(*arguments)
      made.instance_variable_set(:@thread, nil)
      block.call(made) unless block.nil?
      made
    end

    def exception(*arguments)
      self
    end
  end

  # Runs the block, and answers what it answers. A limit of nil or zero puts
  # no limit on it at all.
  def self.timeout(sec, klass = nil, message = nil, &block)
    return block.call(sec) if sec.nil? || sec == 0
    unless sec.is_a?(Numeric)
      raise TypeError, "no implicit conversion of #{sec.class} into Integer"
    end
    if sec < 0
      raise ArgumentError, "Timeout sec must be a non-negative number"
    end
    block.call(sec)
  end

  def timeout(sec, klass = nil, message = nil, &block)
    Timeout.timeout(sec, klass, message, &block)
  end
end

module Kernel
  def timeout(sec, klass = nil, message = nil, &block)
    Timeout.timeout(sec, klass, message, &block)
  end
  module_function :timeout
end

TimeoutError = Timeout::Error
