# FileUtils carries every command MRI's does, each a module function a class
# can include, printing the shell command it stands for when asked to be
# verbose, and the Verbose, NoWrite and DryRun modules that always are, or
# never touch the files.
require "fileutils"
require "tmpdir"

def tree(root)
  Dir.glob("**/*", File::FNM_DOTMATCH, base: root).reject { |name| name.end_with?(".") }.sort.map do |name|
    path = File.join(root, name)
    if File.symlink?(path)
      "#{name} -> #{File.readlink(path)}"
    elsif File.directory?(path)
      "#{name}/ #{'%o' % (File.stat(path).mode & 07777)}"
    else
      "#{name} #{'%o' % (File.stat(path).mode & 07777)} #{File.read(path).inspect} links=#{File.stat(path).nlink}"
    end
  end
end

def attempt(label)
  answer = yield
  puts "#{label} => #{answer.inspect}"
rescue StandardError => error
  puts "#{label} !! #{error.class}: #{error.message.sub(Dir.pwd, '.')}"
end

Dir.mktmpdir do |root|
  Dir.chdir(root) do
    attempt("mkdir") { FileUtils.mkdir("a", verbose: true) }
    attempt("mkdir mode") { FileUtils.mkdir(%w[b c], mode: 0750, verbose: true) }
    attempt("mkdir_p") { FileUtils.mkdir_p("a/b/c/", verbose: true) }
    attempt("mkdir_p noop") { FileUtils.mkdir_p(["x/y", "z"], noop: true, verbose: true) }
    attempt("mkdir exists") { FileUtils.mkdir("a") }
    File.write("a/one.txt", "one")
    File.write("a/b/two.txt", "two")
    attempt("touch") { FileUtils.touch(["t1", "t2"], verbose: true) }
    attempt("touch nocreate") { FileUtils.touch("missing", nocreate: true, verbose: true) }
    attempt("touch mtime") { FileUtils.touch("t1", mtime: Time.at(1_000_000_000).utc, verbose: true) }
    p File.mtime("t1").to_i
    attempt("cp") { FileUtils.cp("a/one.txt", "copy.txt", verbose: true) }
    attempt("cp into dir") { FileUtils.cp(%w[a/one.txt t1], "c", verbose: true) }
    attempt("cp same") { FileUtils.cp("copy.txt", "copy.txt") }
    attempt("cp_r") { FileUtils.cp_r("a", "a2", verbose: true) }
    attempt("cp_r preserve") { FileUtils.cp_r("a", "c", preserve: true, verbose: true) }
    attempt("cp_lr") { FileUtils.cp_lr("a", "a3", verbose: true) }
    attempt("ln") { FileUtils.ln("copy.txt", "hard.txt", verbose: true) }
    attempt("ln force") { FileUtils.ln("t2", "hard.txt", force: true, verbose: true) }
    attempt("ln_s") { FileUtils.ln_s("copy.txt", "soft.txt", verbose: true) }
    attempt("ln_s exists") { FileUtils.ln_s("copy.txt", "soft.txt") }
    attempt("ln_sf") { FileUtils.ln_sf("t1", "soft.txt", verbose: true) }
    attempt("ln_sr") { FileUtils.ln_sr("a/b/two.txt", "c", verbose: true) }
    attempt("ln_s relative") { FileUtils.ln_s("a/one.txt", "a2/b/rel.txt", relative: true, verbose: true) }
    attempt("mv") { FileUtils.mv("t2", "moved", verbose: true) }
    attempt("mv into dir") { FileUtils.mv(%w[moved copy.txt], "b", verbose: true) }
    attempt("mv onto dir") { FileUtils.mv("a3", "a2") }
    attempt("mv missing force") { FileUtils.mv("nope", "nowhere", force: true, verbose: true) }
    attempt("chmod") { FileUtils.chmod(0600, "t1", verbose: true) }
    attempt("chmod symbolic") { FileUtils.chmod("u+x,go=r", "b/moved", verbose: true) }
    attempt("chmod X") { FileUtils.chmod("a-x,a+X", "a2", verbose: true) }
    attempt("chmod bad") { FileUtils.chmod("q+x", "t1") }
    attempt("chmod_R") { FileUtils.chmod_R("g+w", "a2", verbose: true) }
    attempt("install") { FileUtils.install("t1", "inst/deep/t1", mode: 0640, verbose: true) }
    attempt("install again") { FileUtils.install("t1", "inst/deep/t1", mode: "u+x", preserve: true, verbose: true) }
    attempt("install dir") { FileUtils.install("t1", "inst2/", verbose: true) }
    attempt("compare_file") { [FileUtils.compare_file("t1", "inst/deep/t1"), FileUtils.cmp("t1", "a/one.txt"), FileUtils.identical?("a/one.txt", "a2/one.txt")] }
    attempt("uptodate?") { [FileUtils.uptodate?("a/one.txt", ["t1"]), FileUtils.uptodate?("t1", ["a/one.txt"]), FileUtils.uptodate?("none", [])] }
    attempt("copy_entry") { FileUtils.copy_entry("a", "entry") }
    attempt("copy_file") { FileUtils.copy_file("a/one.txt", "file_copy") }
    attempt("copy_stream") { File.open("a/one.txt") { |source| File.open("streamed", "w") { |target| FileUtils.copy_stream(source, target) } } }
    attempt("chown self") { FileUtils.chown(nil, nil, "t1", verbose: true) }
    puts tree(".")
    attempt("rm") { FileUtils.rm(%w[streamed file_copy], verbose: true) }
    attempt("rm missing") { FileUtils.rm("nope") }
    attempt("rm_f") { FileUtils.rm_f(%w[nope t1], verbose: true) }
    attempt("rm_r") { FileUtils.rm_r("entry", verbose: true) }
    attempt("rm_rf") { FileUtils.rm_rf(%w[a3 a2 none], verbose: true) }
    attempt("rmdir") { FileUtils.rmdir("c", verbose: true) }
    attempt("rmdir parents") { FileUtils.mkdir_p("p/q/r"); FileUtils.rmdir("p/q/r", parents: true, verbose: true) }
    attempt("remove_dir") { FileUtils.remove_dir("inst2") }
    attempt("remove_dir file") { FileUtils.remove_dir("soft.txt") }
    attempt("remove_entry_secure") { FileUtils.remove_entry_secure("inst") }
    attempt("remove_file") { FileUtils.remove_file("hard.txt") }
    attempt("cd") { FileUtils.cd("a", verbose: true) { FileUtils.pwd.end_with?("/a") } }
    puts tree(".")
    attempt("Verbose") { FileUtils::Verbose.touch("v1") }
    attempt("NoWrite") { FileUtils::NoWrite.rm("v1") }
    attempt("DryRun") { FileUtils::DryRun.rm_rf("a") }
    attempt("DryRun cp") { FileUtils::DryRun.cp("v1", "v2") }
    p File.exist?("v1"), File.exist?("v2"), File.directory?("a")
    attempt("NoWrite pwd") { FileUtils::NoWrite.pwd }
    attempt("label") do
      FileUtils.instance_variable_set(:@fileutils_label, "step: ")
      FileUtils.touch("labelled", verbose: true)
    ensure
      FileUtils.remove_instance_variable(:@fileutils_label)
    end
    including = Class.new do
      include FileUtils
      def go = (mkdir_p "inc/one"; touch "inc/one/file"; cp_r "inc", "inc2"; Dir.glob("inc2/**/*").sort)
    end
    attempt("included") { including.new.go }
    attempt("private") { including.new.respond_to?(:mkdir) }
  end
end
