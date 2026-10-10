# Where a gem's files would live. Metorex ships one binary rather than an
# installed tree, so nothing is found under any of these names.

module Gem
  class LoadError < ::LoadError
  end

  class MissingSpecError < LoadError
  end

  def self.dir
    File.join Dir.home, ".gem", "metorex"
  end

  def self.path
    [dir]
  end

  def self.default_dir
    dir
  end

  def self.default_specifications_dir
    File.join default_dir, "specifications", "default"
  end

  def self.bindir
    File.join dir, "bin"
  end

  def self.bin_path name, executable = nil, *_requirements
    raise MissingSpecError, "can't find gem #{name}" if executable.nil?
    File.join bindir, executable
  end

  # A gem's files go in front of the ones the interpreter carries, which is
  # what the index names.
  def self.load_path_insert_index
    $LOAD_PATH.length
  end

  def self.loaded_specs
    {}
  end

  # Whether a gem holding `path` was activated to put it on the load path.
  # No gem is installed under `dir`, so none is.
  def self.try_activate _path
    false
  end

  def self.ruby
    RbConfig.ruby
  end

  # The settings a gem command reads. Metorex keeps no configuration file,
  # so each setting starts at the default RubyGems gives it.
  class ConfigFile
    DEFAULT_BACKTRACE = true
    DEFAULT_VERBOSITY = true

    attr_accessor :verbose, :backtrace

    def initialize(_arguments = [])
      @verbose = DEFAULT_VERBOSITY
      @backtrace = DEFAULT_BACKTRACE
    end

    # Whether every detail is wanted, which a verbosity other than true,
    # false, or nil asks for.
    def really_verbose
      case verbose
      when true, false, nil then
        false
      else
        true
      end
    end
  end

  def self.configuration
    @configuration ||= Gem::ConfigFile.new
  end

  def self.configuration=(config)
    @configuration = config
  end

  class Specification
    def self.default_specifications_dir
      Gem.default_specifications_dir
    end

    def self.each_spec _directories
      nil
    end

    def self.find_by_name name, *_requirements
      raise MissingSpecError, "can't find gem #{name}"
    end

    # The newest version of each installed gem, of which there are none.
    def self.latest_specs _prerelease = false
      []
    end
  end
end

require 'rbconfig'
