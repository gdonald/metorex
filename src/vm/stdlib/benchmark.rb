# Timing a block: the processor time it took in user and system mode, for
# the process and its children, and the time that passed on the clock.
module Benchmark
  VERSION = "0.5.0"
  BENCHMARK_VERSION = "2002-04-25"

  # One measurement, and the arithmetic that combines measurements.
  class Tms
    CAPTION = "      user     system      total        real\n"
    FORMAT = "%10.6u %10.6y %10.6t %10.6r\n"

    attr_reader :utime, :stime, :cutime, :cstime, :real, :total, :label

    def initialize(utime = 0.0, stime = 0.0, cutime = 0.0, cstime = 0.0, real = 0.0, label = nil)
      @utime = utime
      @stime = stime
      @cutime = cutime
      @cstime = cstime
      @real = real
      @label = label.to_s
      @total = @utime + @stime + @cutime + @cstime
    end

    # This measurement plus one of the block.
    def add(&block)
      self + Benchmark.measure(&block)
    end

    # Add a measurement of the block to this one in place.
    def add!(&block)
      measured = Benchmark.measure(&block)
      @utime += measured.utime
      @stime += measured.stime
      @cutime += measured.cutime
      @cstime += measured.cstime
      @real += measured.real
      self
    end

    def +(other)
      memberwise(:+, other)
    end

    def -(other)
      memberwise(:-, other)
    end

    def *(other)
      memberwise(:*, other)
    end

    def /(other)
      memberwise(:/, other)
    end

    # The measurement written through `format`, where %u, %y, %U, %Y, %t and
    # %r stand for the user, system, children's user, children's system,
    # total and real times, %n for the label, and any width and precision
    # written in them apply. Arguments given fill the rest of the format.
    def format(format = nil, *arguments)
      written = (format || FORMAT).dup
      written = written.gsub(/(%[-+.\d]*)n/) { "#{$1}s" % label }
      written = written.gsub(/(%[-+.\d]*)u/) { "#{$1}f" % utime }
      written = written.gsub(/(%[-+.\d]*)y/) { "#{$1}f" % stime }
      written = written.gsub(/(%[-+.\d]*)U/) { "#{$1}f" % cutime }
      written = written.gsub(/(%[-+.\d]*)Y/) { "#{$1}f" % cstime }
      written = written.gsub(/(%[-+.\d]*)t/) { "#{$1}f" % total }
      written = written.gsub(/(%[-+.\d]*)r/) { "(#{$1}f)" % real }
      format ? written % arguments : written
    end

    def to_s
      format
    end

    def to_a
      [@label, @utime, @stime, @cutime, @cstime, @real]
    end

    def to_h
      { label: @label, utime: @utime, stime: @stime, cutime: @cutime, cstime: @cstime, real: @real }
    end

    protected

    # Each time combined with the same time of another measurement, or with
    # one number.
    def memberwise(operator, other)
      if other.is_a?(Benchmark::Tms)
        Benchmark::Tms.new(
          utime.__send__(operator, other.utime),
          stime.__send__(operator, other.stime),
          cutime.__send__(operator, other.cutime),
          cstime.__send__(operator, other.cstime),
          real.__send__(operator, other.real)
        )
      else
        Benchmark::Tms.new(
          utime.__send__(operator, other),
          stime.__send__(operator, other),
          cutime.__send__(operator, other),
          cstime.__send__(operator, other),
          real.__send__(operator, other)
        )
      end
    end
  end

  CAPTION = Tms::CAPTION
  FORMAT = Tms::FORMAT

  # What `bmbm` collects: each label with its block, run later.
  class Job
    attr_reader :list, :width

    def initialize(width)
      @width = width
      @list = []
    end

    def item(label = "", &block)
      raise ArgumentError, "no block" unless block
      label = label.to_s
      @width = label.length if @width < label.length
      @list << [label, block]
      self
    end

    alias report item
  end

  # What `benchmark` hands its block: each report measures its block at once.
  class Report
    attr_reader :width, :format, :list

    def initialize(width = 0, format = nil)
      @width = width
      @format = format
      @list = []
    end

    def item(label = "", *_format, &block)
      @width = label.to_s.length if @width < label.to_s.length
      measured = Benchmark.measure(label, &block)
      @list << measured
      measured
    end

    alias report item
  end

  # Print a caption and a line for each report the block makes, then a line
  # for each measurement in an Array the block answers, labelled by
  # `labels` in turn.
  def benchmark(caption = "", label_width = nil, format = nil, *labels)
    sync = $stdout.sync
    $stdout.sync = true
    label_width = (label_width || 0) + 1
    format ||= FORMAT
    report = Report.new(label_width, format)
    results = yield(report)
    print " " * report.width + caption unless caption.empty?
    report.list.each do |measured|
      print measured.label.to_s.ljust(report.width)
      print measured.format(report.format, *format)
    end
    if results.is_a?(Array)
      results.grep(Tms).each do |measured|
        print((labels.shift || measured.label || "").ljust(label_width), measured.format(format))
      end
    end
    report.list
  ensure
    $stdout.sync = sync unless sync.nil?
  end

  def bm(label_width = 0, *labels, &block)
    benchmark(CAPTION, label_width, FORMAT, *labels, &block)
  end

  # Run every block twice, first as a rehearsal whose total is printed, then
  # for the measurements answered, so the second run is not the one paying
  # for whatever the first run set up.
  def bmbm(width = 0)
    job = Job.new(width)
    yield(job)
    width = job.width + 1
    sync = $stdout.sync
    $stdout.sync = true
    puts "Rehearsal ".ljust(width + CAPTION.length, "-")
    rehearsed = job.list.inject(Tms.new) do |sum, (label, block)|
      print label.ljust(width)
      measured = Benchmark.measure(&block)
      print measured.format
      sum + measured
    end
    total = rehearsed.format("total: %tsec")
    print " #{total}\n\n".rjust(width + CAPTION.length + 2, "-")
    print " " * width + CAPTION
    job.list.map do |label, block|
      GC.start
      print label.ljust(width)
      measured = Benchmark.measure(label, &block)
      print measured
      measured
    end
  ensure
    $stdout.sync = sync unless sync.nil?
  end

  def measure(label = "")
    before = Process.times
    started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    yield
    after = Process.times
    finished = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    Benchmark::Tms.new(
      after.utime - before.utime,
      after.stime - before.stime,
      after.cutime - before.cutime,
      after.cstime - before.cstime,
      finished - started,
      label
    )
  end

  # The seconds the block took on the clock.
  def realtime
    started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    yield
    Process.clock_gettime(Process::CLOCK_MONOTONIC) - started
  end

  # The milliseconds the block took on the clock.
  def ms
    started = Process.clock_gettime(Process::CLOCK_MONOTONIC, :float_millisecond)
    yield
    Process.clock_gettime(Process::CLOCK_MONOTONIC, :float_millisecond) - started
  end

  module_function :benchmark, :measure, :realtime, :ms, :bm, :bmbm
end
