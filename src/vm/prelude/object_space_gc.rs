pub(super) const SOURCE: &str = r##"
module ObjectSpace
  # The finalizers a program has asked for, under the id of the object each
  # one belongs to. Metorex frees an object when its last reference goes and
  # nothing watches for that, so a finalizer runs as the program ends.
  def self.__finalizers__
    @finalizers = {} if @finalizers.nil?
    @finalizers
  end

  def self.define_finalizer(held, callable = nil, &block)
    finalizer = callable.nil? ? block : callable
    raise ArgumentError, "no finalizer given" if finalizer.nil?
    unless finalizer.respond_to? :call
      raise ArgumentError, "wrong type argument #{finalizer.class} (should be callable)"
    end
    case held
    when nil, true, false, Symbol, Integer, Float
      raise ArgumentError, "cannot define finalizer for #{held.class}"
    end
    if __finalizer_receiver__(finalizer).equal? held
      warn "finalizer references object to be finalized", uplevel: 1
    end
    named = held.object_id
    listed = ObjectSpace.__finalizers__.fetch(named, [])
    same = listed.find { |one| one == finalizer }
    return [0, same] unless same.nil?
    ObjectSpace.__finalizers__[named] = listed + [finalizer]
    unless @armed
      @armed = true
      at_exit { ObjectSpace.__run_finalizers__ }
    end
    [0, finalizer]
  end

  # The object a finalizer runs as: a block's `self`, or the object a method
  # was taken from.
  def self.__finalizer_receiver__(finalizer)
    return finalizer.binding.receiver if finalizer.is_a? Proc
    return finalizer.receiver if finalizer.is_a? Method
    nil
  end

  # Run every finalizer as the program ends, including the ones a finalizer
  # defines while it runs. One that raises is reported unless warnings are
  # switched off, and the rest still run.
  def self.__run_finalizers__
    until __finalizers__.empty?
      pending = __finalizers__.to_a
      __finalizers__.clear
      pending.each do |id, listed|
        listed.each do |one|
          begin
            one.call id
          rescue Exception => error
            unless $VERBOSE.nil?
              warn "warning: Exception in finalizer #{one.inspect}"
              $stderr.write error.full_message(highlight: false)
            end
          end
        end
      end
    end
  end

  # A clone carries the finalizers its original was given, which is what
  # makes both of them run one as the program ends.
  def self.__carry_finalizers__(from, to)
    listed = __finalizers__[from.object_id]
    return nil if listed.nil?
    __finalizers__[to.object_id] = __finalizers__.fetch(to.object_id, []) + listed
    nil
  end

  def self.undefine_finalizer(held)
    if held.frozen?
      raise FrozenError, "can't modify frozen #{held.class}: #{held.inspect}"
    end
    ObjectSpace.__finalizers__.delete held.object_id
    held
  end

  # Hand every object still alive that is a kind of `wanted` to the block,
  # answering how many there were.
  def self.each_object(wanted = nil)
    unless wanted.nil? || wanted.is_a?(Module)
      raise TypeError, "class or module required"
    end
    return to_enum(:each_object, wanted) unless block_given?
    found = __each_object__(wanted)
    found.each { |one| yield one }
    found.size
  end

  # Ruby 4.0 deprecated reading an object back from its id. Metorex keeps no
  # table of every live object, so only the values whose id is derived from
  # the value itself can be answered at all.
  def self._id2ref(id)
    warn "ObjectSpace._id2ref is deprecated", uplevel: 1, category: :deprecated
    return nil if id == 4
    return true if id == 2
    return false if id == 0
    return (id - 1) / 2 if id.odd? || id < 0
    raise RangeError, "#{id} is not an id value"
  end
end

module GC
  # The collector's settings. `:implementation` names the collector and is
  # read-only, and the rest are whatever the collector takes. Metorex frees
  # an object when its last reference goes, so it takes none.
  def self.config(options = :__none__)
    @settings = {} if @settings.nil?
    current = { implementation: "metorex" }.merge(@settings)
    return current if options == :__none__ || options.nil?
    unless options.is_a? Hash
      raise ArgumentError, "expecting a Hash, got #{options.class}"
    end
    if options.key? :implementation
      raise ArgumentError, 'Attempting to set read-only key "Implementation"'
    end
    current
  end

  # An object that extends GC answers `garbage_collect` the way the module
  # itself does, and Ruby documents the answer as always nil.
  def garbage_collect(full_mark: true, immediate_sweep: true)
    GC.start
    nil
  end

  # Metorex frees an object when its last reference goes, so switching the
  # collector off leaves nothing running differently. The switch is still read
  # back the way it was written, which is what a program that saves and
  # restores it depends on.
  def self.disable
    was = @disabled == true
    @disabled = true
    was
  end

  def self.enable
    was = @disabled == true
    @disabled = false
    was
  end

  # The readings Ruby's collector reports. Metorex frees an object when the
  # last reference to it goes, so what these count is the work the program
  # has asked for rather than a collector's own bookkeeping.
  def self.stat(target = nil)
    readings = __stat_readings__
    return readings if target.nil?
    if target.is_a? Symbol
      raise ArgumentError, "unknown key: #{target}" unless readings.key?(target)
      return readings[target]
    end
    raise TypeError, "non-hash or symbol given" unless target.is_a? Hash
    readings.each { |name, reading| target[name] = reading }
    target
  end

  def self.__stat_readings__
    runs = GC.count
    counted = ObjectSpace.count_objects
    live = counted.nil? ? 1 : counted[:TOTAL].to_i
    {
      count: runs,
      time: GC.total_time,
      marking_time: 0,
      sweeping_time: 0,
      heap_allocated_pages: 1,
      heap_sorted_length: 1,
      heap_allocatable_pages: 0,
      heap_available_slots: live,
      heap_live_slots: live,
      heap_free_slots: 0,
      heap_final_slots: 0,
      heap_marked_slots: live,
      heap_eden_pages: 1,
      heap_tomb_pages: 0,
      total_allocated_pages: 1,
      total_freed_pages: 0,
      total_allocated_objects: live,
      total_freed_objects: 0,
      malloc_increase_bytes: 0,
      malloc_increase_bytes_limit: 0,
      minor_gc_count: 0,
      major_gc_count: runs,
      compact_count: 0,
      read_barrier_faults: 0,
      total_moved_objects: 0,
      remembered_wb_unprotected_objects: 0,
      remembered_wb_unprotected_objects_limit: 0,
      old_objects: 0,
      old_objects_limit: 0,
      oldmalloc_increase_bytes: 0,
      oldmalloc_increase_bytes_limit: 0
    }
  end

  def self.auto_compact
    @auto_compact == true
  end

  def self.auto_compact=(wanted)
    @auto_compact = wanted
  end

  def self.measure_total_time
    @measure_total_time == true
  end

  def self.measure_total_time=(wanted)
    @measure_total_time = wanted
  end

  # Ruby collects on every allocation while this is set, which is a way to
  # shake out collector bugs. There is no collector here to drive that hard,
  # so the switch is read back the way it was written and nothing else.
  def self.stress
    @stress == true
  end

  def self.stress=(wanted)
    @stress = wanted
  end

  # Ruby's collector reports what each run cost when the profiler is enabled.
  # There are no runs to report here, so the report stays empty however the
  # switch is set.
  module Profiler
    def self.enabled?
      @enabled == true
    end

    def self.enable
      @enabled = true
      nil
    end

    def self.disable
      @enabled = false
      nil
    end

    def self.clear
      nil
    end

    def self.result
      ""
    end

    def self.report(target = nil)
      nil
    end

    def self.total_time
      0.0
    end
  end
end

module Process
  # Watch a child in a thread of its own, so the program does not have to
  # wait on it. The thread answers the status the child exited with.
  def self.detach(pid)
    watched = __pid_argument__ pid
    watching = Thread.new do
      begin
        Process.waitpid watched
        $?
      rescue SystemCallError
        nil
      end
    end
    watching[:pid] = watched
    watching.define_singleton_method(:pid) { watched }
    watching
  end

  # The process id an argument names. Anything that reads as an Integer
  # names one, and anything else is refused.
  def self.__pid_argument__(pid)
    return pid if pid.is_a? Integer
    unless pid.respond_to? :to_int
      raise TypeError, "no implicit conversion of #{pid.class} into Integer"
    end
    held = pid.to_int
    unless held.is_a? Integer
      raise TypeError,
            "can't convert #{pid.class} into Integer (#{pid.class}#to_int gives #{held.class})"
    end
    held
  end

  # The four processor-time readings `Process.times` reports: this process's
  # own user and system time, and the totals for the children it waited for.
  Tms = Struct.new(:utime, :stime, :cutime, :cstime)
end

"##;
