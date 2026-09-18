# Coverage measurement. The counting itself is done as each file is read and
# as each line runs, and the bookkeeping here decides what a run reports.
module Coverage
  MODES = [:lines, :branches, :methods, :eval, :oneshot_lines]

  # The modes a run reports under their own names, in the order Ruby lists
  # them.
  REPORTED_MODES = [:lines, :branches, :methods]

  module_function

  def supported?(mode)
    unless mode.is_a?(Symbol)
      raise TypeError, "wrong argument type #{mode.class} (expected Symbol)"
    end
    MODES.include?(mode)
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

    if options == :all
      modes = REPORTED_MODES
      by_mode = true
      eval_too = true
    elsif options.is_a?(Hash)
      modes = REPORTED_MODES.select { |mode| options[mode] }
      by_mode = !modes.empty? || options[:oneshot_lines] ? true : false
      modes = [:lines] if modes.empty?
      eval_too = options[:eval] ? true : false
    else
      modes = [:lines]
      by_mode = false
      eval_too = false
    end

    @running = true
    __coverage_start__ by_mode, modes, eval_too
    nil
  end

  def setup(options = nil)
    start options
  end

  def result(**options)
    raise RuntimeError, "coverage measurement is not enabled" unless running?
    stop_given = options.key? :stop
    clear_given = options.key? :clear
    stop = stop_given ? options[:stop] : !clear_given
    clear = clear_given ? options[:clear] : stop
    if stop && stop_given && !(clear_given && clear)
      warn "warning: stop implies clear"
      clear = true
    end
    answer = __coverage_result__ clear
    if stop
      @running = false
      __coverage_stop__
    end
    answer
  end

  def peek_result
    raise RuntimeError, "coverage measurement is not enabled" unless running?
    __coverage_result__ false
  end

  def line_stub(path)
    File.readlines(path).map { nil }
  end
end
