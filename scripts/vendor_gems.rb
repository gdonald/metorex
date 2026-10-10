# Copies the bundled gems MRI ships that metorex carries into this repository.
# Each gem's Ruby library goes into src/vm/stdlib/gems, which `require` finds
# by feature name, and nkf's C extension into vendor/nkf, which build.rs
# compiles into the binary. MRI's resolv.rb, which resolv-replace and rinda's
# ring build on, and its pp.rb and prettyprint.rb, which irb prints with,
# replace metorex's own.
#
#   ruby scripts/vendor_gems.rb
#
# The gems are the ones the running Ruby would load.

require "fileutils"

GEMS = %w[irb mutex_m nkf racc reline resolv-replace rinda].freeze

root = File.expand_path("..", __dir__)
library = File.join(root, "src", "vm", "stdlib", "gems")
FileUtils.rm_rf(library)

entries = []
versions = []
GEMS.each do |name|
  gem_dir = Gem::Specification.find_by_name(name).gem_dir
  version = File.basename(gem_dir).delete_prefix("#{name}-")
  versions << "#{name} #{version}"
  lib = File.join(gem_dir, "lib")
  license = %w[LICENSE.txt BSDL COPYING].map { |file| File.join(gem_dir, file) }.select { |path| File.exist?(path) }
  license.each do |path|
    FileUtils.mkdir_p(File.join(library, name))
    FileUtils.cp(path, File.join(library, name, File.basename(path)))
  end
  Dir.glob("**/*.rb", base: lib).sort.each do |path|
    text = File.read(File.join(lib, path))
    # An embedded library is required by feature name, so a relative require
    # names the feature it resolves to.
    text = text.gsub(/require_relative ["']([^"']+)["']/) do
      feature = File.expand_path($1, File.join("/", File.dirname(path))).delete_prefix("/")
      %(require "#{feature}")
    end
    destination = File.join(library, name, path)
    FileUtils.mkdir_p(File.dirname(destination))
    File.write(destination, text)
    entries << %(    ("#{path.delete_suffix(".rb")}", include_str!("gems/#{name}/#{path}")),\n)
  end
end

# irb finds its message files along the load path, where metorex has none of
# its own: they are carried with the rest of irb, so irb is told to look there.
File.open(File.join(library, "irb", "irb", "locale.rb"), "a") do |locale|
  locale.write(<<~'RUBY')

    # Added by metorex's scripts/vendor_gems.rb: irb's message files are carried
    # inside the interpreter with the rest of irb, and found there by name.
    module IRB
      class Locale
        alias_method :__search_file_on_disk__, :search_file
        def search_file(lib_paths, dir, file)
          each_localized_path(dir, file) do |lc_path|
            if __embedded_library_names__().include?(lc_path.delete_suffix(".rb"))
              return "<metorex>/#{lc_path}"
            end
          end
          __search_file_on_disk__(lib_paths, dir, file)
        end

        alias_method :__load_from_disk__, :load
        def load(file)
          found = find(file)
          return __load_from_disk__(file) unless found&.start_with?("<metorex>/")
          require found.delete_prefix("<metorex>/").delete_suffix(".rb")
        end
      end
    end
  RUBY
end

File.write(File.join(root, "src", "vm", "stdlib", "gem_libraries.rs"), <<~RUST)
  // Written by scripts/vendor_gems.rb from #{versions.join(", ")}.

  /// The files of the bundled gems metorex carries, by the feature name each
  /// is required as.
  pub(crate) const GEM_LIBRARIES: &[(&str, &str)] = &[
  #{entries.join.chomp}
  ];
RUST

# Libraries MRI writes in Ruby that metorex carries as MRI has them: its own
# resolv.rb, io/console/size.rb and pathname.rb, and pp.rb and prettyprint.rb
# from the gems `pp` loads.
{
  File.join(RbConfig::CONFIG["rubylibdir"], "resolv.rb") => "resolv.rb",
  File.join(Gem::Specification.find_by_name("pp").gem_dir, "lib", "pp.rb") => "pp.rb",
  File.join(Gem::Specification.find_by_name("prettyprint").gem_dir, "lib", "prettyprint.rb") => "prettyprint.rb",
  File.join(RbConfig::CONFIG["rubylibdir"], "io", "console", "size.rb") => "io_console_size.rb",
  File.join(RbConfig::CONFIG["rubylibdir"], "pathname.rb") => "pathname.rb",
}.each do |source, name|
  FileUtils.cp(source, File.join(root, "src", "vm", "stdlib", name))
end

# Pathname's methods, which MRI compiles in from pathname_builtin.rb rather
# than installing, so they are read from MRI's source at the running Ruby's
# revision.
require "open-uri"
builtin = "https://raw.githubusercontent.com/ruby/ruby/#{RUBY_REVISION}/pathname_builtin.rb"
File.write(File.join(root, "src", "vm", "stdlib", "pathname_builtin.rb"), URI.open(builtin, &:read))

# nkf's extension: its Ruby binding and the nkf library it includes.
nkf_ext = File.join(Gem::Specification.find_by_name("nkf").gem_dir, "ext", "nkf")
vendor = File.join(root, "vendor", "nkf")
FileUtils.rm_rf(vendor)
FileUtils.mkdir_p(File.join(vendor, "nkf-utf8"))
FileUtils.cp(File.join(nkf_ext, "nkf.c"), vendor)
Dir.glob(File.join(nkf_ext, "nkf-utf8", "*.{c,h}")).each { |path| FileUtils.cp(path, File.join(vendor, "nkf-utf8")) }
FileUtils.cp(File.join(Gem::Specification.find_by_name("nkf").gem_dir, "LICENSE.txt"), vendor)

puts "#{versions.join(", ")}: #{entries.size} Ruby files"
