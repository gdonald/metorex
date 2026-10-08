# mkmf's checks compile a small program to learn whether a header, library,
# function, variable, type, macro or constant is there, print what they
# checked for, and record what they found for the Makefile to carry.
# `ruby -run -e mkmf` runs them from the command line.
require("mkmf")
require("tmpdir")
require("rbconfig")

Dir.mktmpdir do |root|
  Dir.chdir(root) do
    p(have_header("stdio.h"))
    p(have_header("no_such_header.h"))
    p(have_library("m", "sqrt"))
    p(have_library("no_such_library_xyz", "nothing"))
    p(have_func("printf", "stdio.h"))
    p(have_func("no_such_function_xyz"))
    p(have_func("strlen", ["stdio.h", "string.h"]))
    p(have_var("errno", "errno.h"))
    p(have_var("no_such_variable_xyz", "stdio.h"))
    p(have_type("size_t", "stddef.h"))
    p(have_type("struct no_such_type", "stdio.h"))
    p(have_macro("EOF", "stdio.h"))
    p(have_macro("NO_SUCH_MACRO_XYZ", "stdio.h"))
    p(have_const("EOF", "stdio.h"))
    p(have_const("NO_SUCH_CONST_XYZ", "stdio.h"))
    p(dir_config("probe_lib"))
    p(dir_config("other", "/def/inc", "/def/lib"))
    p($defs)
    p($libs)
    p($CPPFLAGS.split.grep(/\A-I/), $LIBPATH)
    p("my-lib.h".tr_cpp, "foo".funcall_style, "foo(1)".sans_arguments)
    File.write("probe.c", "#include \"ruby.h\"\nvoid Init_probe(void) {}\n")
    create_makefile("probe")
    p(File.read("Makefile").scan(/-DHAVE_\w+|-lm\b/).uniq.sort)

    File.delete("Makefile")
    read, write = IO.pipe
    child = spawn(RbConfig.ruby, "-run", "-e", "mkmf", "--", "-h", "stdio.h:no_such.h", "-l", "m,sqrt",
                  "-f", "printf,stdio.h", "-t", "size_t,stddef.h", "-c", "EOF,stdio.h", "probe",
                  out: write, err: write)
    write.close
    puts(read.read)
    Process.wait(child)
    p(File.read("Makefile").scan(/-DHAVE_\w+|-lm\b/).uniq.sort)
  end
end
