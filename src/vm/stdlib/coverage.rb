# Coverage measurement. The modes are reported as unsupported, since metorex
# records nothing yet, and the start and stop bookkeeping is kept so a caller
# can ask whether measurement is on.
module Coverage
  MODES = [:lines, :branches, :methods, :eval, :oneshot_lines]

  module_function

  def supported?(mode)
    unless mode.is_a?(Symbol)
      raise TypeError, "wrong argument type #{mode.class} (expected Symbol)"
    end
    false
  end

  def running?
    @running ? true : false
  end

  def start(options = nil)
    raise RuntimeError, "coverage measurement is already setup" if running?
    unless options.nil? || options == :all || options.is_a?(Hash)
      raise TypeError, "no implicit conversion of #{options.class} into Hash"
    end
    if options.is_a?(Hash) && options[:lines] && options[:oneshot_lines]
      raise RuntimeError, "cannot enable lines and oneshot_lines simultaneously"
    end
    @running = true
    nil
  end

  def setup(options = nil)
    start(options)
  end

  def result(stop: true, clear: true)
    raise RuntimeError, "coverage measurement is not enabled" unless running?
    @running = false if stop
    {}
  end

  def peek_result
    raise RuntimeError, "coverage measurement is not enabled" unless running?
    {}
  end

  def line_stub(path)
    File.readlines(path).map { nil }
  end
end
