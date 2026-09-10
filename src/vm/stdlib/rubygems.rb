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

  def self.ruby
    RbConfig.ruby
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
  end
end

require 'rbconfig'
