# What the interpreter was built as and where it keeps its parts. Metorex is
# one binary rather than an installed tree, so the directories that only mean
# something after an installation are left out and `TOPDIR` is nil.

module RbConfig
  major, minor, teeny = RUBY_VERSION.split "."
  cpu, os = RUBY_PLATFORM.split "-", 2

  CONFIG = {
    "MAJOR" => major,
    "MINOR" => minor,
    "TEENY" => teeny,
    "PATCHLEVEL" => RUBY_PATCHLEVEL.to_s,
    "ruby_version" => "#{major}.#{minor}.0",
    "RUBY_PROGRAM_VERSION" => RUBY_VERSION.dup,
    "RUBY_INSTALL_NAME" => "metorex",
    "RUBY_BASE_NAME" => "metorex",
    "ruby_install_name" => "metorex",
    "arch" => RUBY_PLATFORM.dup,
    "host" => RUBY_PLATFORM.dup,
    "host_cpu" => cpu,
    "host_os" => os,
    "target" => RUBY_PLATFORM.dup,
    "target_cpu" => cpu,
    "target_os" => os,
    "build" => RUBY_PLATFORM.dup,
    "build_cpu" => cpu,
    "build_os" => os,
    "EXEEXT" => "",
    "DLEXT" => os.include?("darwin") ? "bundle" : "so",
    # The extensions metorex carries are built into the binary, and none is
    # loaded from a file of its own.
    "EXTSTATIC" => "static",
    "LIBEXT" => "a",
    "OBJEXT" => "o",
    "ENABLE_SHARED" => "no",
    "LIBRUBY" => "libmetorex-static.a",
    "LIBRUBY_SO" => "libmetorex.#{os.include?("darwin") ? "dylib" : "so"}",
    "LIBRUBY_A" => "libmetorex-static.a",
    "libdirname" => "libdir",
    "LIBPATHENV" => os.include?("darwin") ? "DYLD_FALLBACK_LIBRARY_PATH" : "LD_LIBRARY_PATH",
    "UNICODE_VERSION" => "17.0.0",
    "UNICODE_EMOJI_VERSION" => "17.0",
    "CC" => "cc",
    "CXX" => "c++",
    "AR" => "ar",
    "STRIP" => "strip",
    "SHELL" => "/bin/sh",
    "EXTOUT" => ".ext",
    "NULLCMD" => ":"
  }

  # A library installed alongside metorex goes under the prefix the binary
  # sits in, which is where `$LOAD_PATH` looks for one.
  prefix = File.dirname(File.dirname(__interpreter_path__()))
  CONFIG["sitelibdir"] = "#{prefix}/lib/metorex/#{RUBY_VERSION}"
  CONFIG["sitearchdir"] = "#{CONFIG["sitelibdir"]}/#{RUBY_PLATFORM}"
  CONFIG["rubyhdrdir"] = __header_directory__()
  CONFIG["rubyarchhdrdir"] = CONFIG["rubyhdrdir"]

  # Metorex runs from wherever its binary sits rather than from an installed
  # tree, so there is no prefix to name.
  TOPDIR = nil

  def self.expand(value, config = CONFIG)
    value
  end

  # The binary running this program, which starts another one.
  def self.ruby
    __interpreter_path__()
  end
end
