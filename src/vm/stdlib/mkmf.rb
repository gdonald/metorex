# The helpers an extconf.rb calls to describe how to build a C extension,
# the checks it runs to learn what the system has, and the Makefile they
# write. The extension is compiled against the headers metorex ships, and the
# `rb_*` functions it calls are left for the metorex binary that loads it to
# supply.

require 'rbconfig'
require 'shellwords'
require 'tmpdir'

class String
  # The name a C preprocessor symbol is spelled with for this text.
  def tr_cpp
    strip.upcase.tr_s("^A-Z0-9_*", "_").tr_s("*", "P")
  end

  def funcall_style
    /\)\z/ =~ self ? dup : "#{self}()"
  end

  # The text in double quotes when it holds whitespace, as a Makefile
  # needs it.
  def quote
    /\s/ =~ self ? "\"#{self}\"" : "#{self}"
  end

  def sans_arguments
    self[/\A[^()]+/]
  end
end

class Array
  def quote
    map(&:quote)
  end
end

module MakeMakefile
  # How a library is named to the linker.
  LIBARG = "-l%s"

  # The libraries every program links against already, which need no check.
  COMMON_LIBS = [].freeze

  # What every source a check compiles starts with.
  COMMON_HEADERS = "#include \"ruby.h\"\n"

  # Whether `message` keeps quiet, which `$VERBOSE` overrides.
  module Logging
    @quiet = $extmk

    class << self
      attr_accessor :quiet
    end
  end

  # A `main` that does nothing, so a source links into a program.
  MAIN_DOES_NOTHING = "int main(int argc, char **argv)\n{\n  return !!argv[argc];\n}\n"

  def append_cflags(flags, *_options)
    $CFLAGS = [$CFLAGS, *flags].compact.join(" ")
    true
  end

  # The value `--name=value` gave on the command line, the default when it
  # was not given, or what the block answers then.
  def arg_config(config, default = nil, &block)
    $arg_config << [config, default]
    defaults = []
    if default
      defaults << default
    elsif !block
      defaults << nil
    end
    $configure_args.fetch(config.tr("_", "-"), *defaults, &block)
  end

  # What `--with-name` or `--without-name` said, with "yes" and "no" read as
  # true and false.
  def with_config(config, default = nil)
    config = config.sub(/^--with[-_]/, "")
    value = arg_config("--with-" + config) do
      if arg_config("--without-" + config)
        false
      elsif block_given?
        yield(config, default)
      else
        break default
      end
    end
    case value
    when "yes" then true
    when "no" then false
    else value
    end
  end

  # The include and library directories `--with-target-dir`,
  # `--with-target-include` and `--with-target-lib` name, or the defaults
  # given, put ahead of the ones searched already.
  def dir_config(target, include_default = nil, lib_default = nil)
    key = [target, include_default, lib_default].compact.join("\0")
    if (configured = $config_dirs[key])
      return configured
    end

    if (dir = with_config(target + "-dir", (include_default unless lib_default)))
      defaults = dir.is_a?(Array) ? dir : dir.split(File::PATH_SEPARATOR)
      include_default = lib_default = nil
    end

    include_dir = with_config(target + "-include", include_default)
    if (given = $arg_config.assoc("--with-#{target}-include"))
      given[1] ||= "${#{target}-dir}/include"
    end
    lib_dir = with_config(target + "-lib", lib_default)
    if (given = $arg_config.assoc("--with-#{target}-lib"))
      given[1] ||= "${#{target}-dir}/lib"
    end

    include_dirs = include_dir ? (include_dir.is_a?(Array) ? include_dir.dup : include_dir.split(File::PATH_SEPARATOR)) : []
    if defaults
      include_dirs.concat(defaults.map { |held| held + "/include" })
      include_dir = ([include_dir] + include_dirs).compact.join(File::PATH_SEPARATOR)
    end
    unless include_dirs.empty?
      flags = include_dirs.map { |held| "-I" + held } - Shellwords.shellwords($CPPFLAGS)
      $CPPFLAGS = (flags.quote << $CPPFLAGS).join(" ") unless flags.empty?
    end

    lib_dirs = lib_dir ? (lib_dir.is_a?(Array) ? lib_dir.dup : lib_dir.split(File::PATH_SEPARATOR)) : []
    if defaults
      lib_dirs.concat(defaults.map { |held| "#{held}/lib" })
      lib_dir = ([lib_dir] + lib_dirs).compact.join(File::PATH_SEPARATOR)
    end
    $LIBPATH = lib_dirs | $LIBPATH

    $config_dirs[key] = [include_dir, lib_dir]
  end

  # Print what is being checked for, run the check, and print its answer.
  def checking_for(subject, format = nil)
    message "%s", "checking for #{subject}... "
    answer = yield
    message "%s\n", format ? format % answer : (answer ? "yes" : "no")
    answer
  end

  def message(*parts)
    return if Logging.quiet && !$VERBOSE

    printf(*parts)
    $stdout.flush
  end

  # What a check says it looks for: the target, the headers it looks in, and
  # the options it compiles with.
  def checking_message(target, place = nil, options = nil)
    [["in", place], ["with", options]].inject(+"#{target}") do |said, (word, noun)|
      if noun
        noun = noun.is_a?(Array) ? noun.join(",") : noun.to_s
        unless noun.empty?
          said << " #{word} " unless said.empty?
          said << noun
        end
      end
      said
    end
  end

  def cpp_include(header)
    return "" unless header
    [*header].map { |held| held.is_a?(String) ? "#include <#{held}>\n" : held }.join
  end

  def append_library(libs, lib)
    format(LIBARG, lib) + " " + libs
  end

  # Whether the source compiles to an object file.
  def try_compile(source, options = "", **_options)
    build_check(source) do |file|
      run_quietly("#{compiler} #{include_flags} #{preprocessor_flags} #{$CFLAGS} #{options} -c #{file} -o conftest.o")
    end
  end

  alias try_header try_compile

  # Whether the source links into a program.
  def try_link(source, options = "", **_options)
    build_check(source) do |file|
      library_paths = $LIBPATH.map { |held| "-L#{held}" }.join(" ")
      run_quietly("#{compiler} -o conftest #{include_flags} #{preprocessor_flags} #{$CFLAGS} #{file} #{library_paths} #{$LDFLAGS} #{options} #{$LIBS}")
    end
  end

  # Whether `func` links, taken by its address and then called.
  def try_func(func, libs, headers = nil, options = "")
    headers = cpp_include(headers)
    prepared = +""
    case func
    when /^&/
      declared = proc { |name| "const volatile void *#{name}" }
    when /\)$/
      strings = []
      call = func.gsub(/""/) do
        named = "s#{strings.size + 1}"
        strings << named
        named
      end
      prepared << "char " << strings.map { |named| %(#{named}[1024] = "") }.join(", ") << "; " unless strings.empty?
    when nil
      call = ""
    else
      call = "#{func}()"
      declared = proc { |name| "void ((*#{name})())" }
    end
    options = options.to_s.empty? ? libs : "#{options} #{libs}"
    taken = declared && try_link(<<~SOURCE, options)
      #{headers}
      extern int t(void);
      #{MAIN_DOES_NOTHING}
      int t(void) { #{declared["volatile p"]}; p = (#{declared[nil]})#{func}; return !p; }
    SOURCE
    taken || (call && try_link(<<~SOURCE, options))
      #{headers}
      extern int t(void);
      #{MAIN_DOES_NOTHING}
      int t(void) { #{prepared}#{call}; return 0; }
    SOURCE
  end

  def try_var(var, headers = nil, options = "")
    try_compile(<<~SOURCE, options)
      #{cpp_include(headers)}
      extern int t(void);
      #{MAIN_DOES_NOTHING}
      int t(void) { const volatile void *volatile p; p = &(&#{var})[0]; return !p; }
    SOURCE
  end

  def try_type(type, headers = nil, options = "")
    return false unless try_compile(<<~SOURCE, options)
      #{cpp_include(headers)}
      typedef #{type} conftest_type;
      int conftestval[sizeof(conftest_type)?1:-1];
    SOURCE
    $defs.push(format("-DHAVE_TYPE_%s", type.tr_cpp))
    true
  end

  def try_const(const, headers = nil, options = "")
    const, type = *const
    return false unless try_compile(<<~SOURCE, options)
      #{cpp_include(headers)}
      typedef #{type || 'int'} conftest_type;
      conftest_type conftestval = #{type ? '' : '(int)'}#{const};
    SOURCE
    $defs.push(format("-DHAVE_CONST_%s", const.tr_cpp))
    true
  end

  def macro_defined?(macro, source, options = "")
    source = source.sub(/[^\n]\z/, "\\&\n")
    try_compile(source + <<~SOURCE, options)
      #ifndef #{macro}
      # error
      |:/ === #{macro} undefined === /:|
      #endif
    SOURCE
  end

  def have_header(header, preheaders = nil, options = "")
    dir_config(header[/.*?(?=\/)|.*?(?=\.)/])
    checking_for header do
      if try_header(cpp_include(preheaders) + cpp_include(header), options)
        $defs.push(format("-DHAVE_%s", header.tr_cpp))
        true
      else
        false
      end
    end
  end

  def have_library(lib, func = nil, headers = nil, options = "")
    dir_config(lib)
    lib = with_config(lib + "lib", lib)
    checking_for checking_message(func && func.funcall_style, LIBARG % lib, options) do
      if COMMON_LIBS.include?(lib)
        true
      else
        libs = append_library($libs, lib)
        if try_func(func, libs, headers, options)
          $libs = libs
          true
        else
          false
        end
      end
    end
  end

  def have_func(func, headers = nil, options = "")
    checking_for checking_message(func.funcall_style, headers, options) do
      if try_func(func, $libs, headers, options)
        $defs << "-DHAVE_#{func.sans_arguments.tr_cpp}"
        true
      else
        false
      end
    end
  end

  def have_var(var, headers = nil, options = "")
    checking_for checking_message(var, headers, options) do
      if try_var(var, headers, options)
        $defs.push(format("-DHAVE_%s", var.tr_cpp))
        true
      else
        false
      end
    end
  end

  def have_type(type, headers = nil, options = "")
    checking_for checking_message(type, headers, options) do
      try_type(type, headers, options)
    end
  end

  def have_macro(macro, headers = nil, options = "")
    checking_for checking_message(macro, headers, options) do
      macro_defined?(macro, cpp_include(headers), options)
    end
  end

  def have_const(const, headers = nil, options = "")
    checking_for checking_message([*const].compact.join(" "), headers, options) do
      try_const(const, headers, options)
    end
  end

  def create_makefile(target, srcprefix = nil)
    source_directory = srcprefix || "."
    sources = Dir.glob(File.join(source_directory, "*.c")).sort
    objects = sources.map { |source| File.basename(source, ".c") + ".o" }
    dlext = RbConfig::CONFIG["DLEXT"]
    link_flags = if RbConfig::CONFIG["host_os"].include?("darwin")
      "-bundle -undefined dynamic_lookup"
    else
      "-shared"
    end
    library_paths = $LIBPATH.map { |held| "-L#{held}" }.join(" ")
    libraries = [$libs, $LIBS].map(&:strip).reject(&:empty?).join(" ")

    makefile = +""
    makefile << "CC = #{RbConfig::CONFIG["CC"]}\n"
    makefile << "CFLAGS = -fPIC -g #{$CFLAGS}\n"
    makefile << "CPPFLAGS = -I#{RbConfig::CONFIG["rubyhdrdir"]} -I#{source_directory} #{[*$defs, preprocessor_flags].join(" ")}".rstrip << "\n"
    makefile << "DLDFLAGS = #{[link_flags, library_paths, $LDFLAGS].map(&:strip).reject(&:empty?).join(" ")}\n"
    makefile << "LIBS = #{libraries}\n"
    makefile << "DLLIB = #{target}.#{dlext}\n"
    makefile << "OBJS = #{objects.join(" ")}\n"
    makefile << "\n"
    makefile << "all: $(DLLIB)\n"
    makefile << "\n"
    makefile << "$(DLLIB): $(OBJS)\n"
    makefile << "\t$(CC) $(DLDFLAGS) -o $@ $(OBJS) $(LIBS)\n"
    sources.zip(objects).each do |source, object|
      makefile << "\n"
      makefile << "#{object}: #{source}\n"
      makefile << "\t$(CC) $(CPPFLAGS) $(CFLAGS) -c #{source} -o #{object}\n"
    end
    message "creating Makefile\n"
    File.write("Makefile", makefile)
    true
  end

  private

  def compiler
    RbConfig::CONFIG["CC"]
  end

  def include_flags
    "-I#{RbConfig::CONFIG["rubyhdrdir"]}"
  end

  # $CPPFLAGS with the Makefile's own variables left out, since a check runs
  # the compiler directly.
  def preprocessor_flags
    $CPPFLAGS.gsub(/\$\(\w+\)/, "").split.join(" ")
  end

  # Write the check's source, with the headers every check starts with, to
  # a directory of its own and answer what the block does with it.
  def build_check(source)
    Dir.mktmpdir("mkmf") do |directory|
      Dir.chdir(directory) do
        File.write("conftest.c", "#{COMMON_HEADERS}\n#{source}")
        yield "conftest.c"
      end
    end
  end

  def run_quietly(command)
    system(command, out: File::NULL, err: File::NULL) ? true : false
  end
end

# The command line's `--name=value` words, as `with_config` reads them.
$configure_args = {}
ARGV.each do |word|
  name, value = word.split("=", 2)
  next unless name
  name = name.tr("_", "-")
  if name.sub!(/\A(?!--)/, "--")
    next unless value
    name.downcase!
  end
  $configure_args[name] = value || true
end

$arg_config = []
$config_dirs = {}
$defs = []
$libs = ""
$LIBPATH = []
$CFLAGS = ""
$CPPFLAGS = ""
$LDFLAGS = ""
$LIBS = ""

include MakeMakefile
