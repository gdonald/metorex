# The helpers an extconf.rb calls to describe how to build a C extension,
# and the Makefile they write. The extension is compiled against the headers
# metorex ships, and the `rb_*` functions it calls are left for the metorex
# binary that loads it to supply.

require 'rbconfig'

module MakeMakefile
  def append_cflags(flags, *_options)
    $CFLAGS = [$CFLAGS, *flags].compact.join(" ")
    true
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

    makefile = +""
    makefile << "CC = #{RbConfig::CONFIG["CC"]}\n"
    makefile << "CFLAGS = -fPIC -g #{$CFLAGS}\n"
    makefile << "CPPFLAGS = -I#{RbConfig::CONFIG["rubyhdrdir"]} -I#{source_directory}\n"
    makefile << "DLDFLAGS = #{link_flags}\n"
    makefile << "DLLIB = #{target}.#{dlext}\n"
    makefile << "OBJS = #{objects.join(" ")}\n"
    makefile << "\n"
    makefile << "all: $(DLLIB)\n"
    makefile << "\n"
    makefile << "$(DLLIB): $(OBJS)\n"
    makefile << "\t$(CC) $(DLDFLAGS) -o $@ $(OBJS)\n"
    sources.zip(objects).each do |source, object|
      makefile << "\n"
      makefile << "#{object}: #{source}\n"
      makefile << "\t$(CC) $(CPPFLAGS) $(CFLAGS) -c #{source} -o #{object}\n"
    end
    File.write("Makefile", makefile)
    true
  end
end

$CFLAGS = ""

include MakeMakefile
