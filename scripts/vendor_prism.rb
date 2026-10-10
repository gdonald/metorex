# Copies the prism gem into this repository: its C parser into vendor/prism,
# which build.rs compiles into the binary, and its Ruby library into
# src/vm/stdlib/prism, which `require "prism"` loads.
#
#   ruby scripts/vendor_prism.rb [path to the prism gem]
#
# Without a path it copies the prism gem the running Ruby would load.

require "fileutils"

root = File.expand_path("..", __dir__)
gem_dir = ARGV[0] || Gem::Specification.find_by_name("prism").gem_dir
abort "no prism gem at #{gem_dir}" unless File.exist?(File.join(gem_dir, "src", "prism.c"))

vendor = File.join(root, "vendor", "prism")
FileUtils.rm_rf(vendor)
FileUtils.mkdir_p(vendor)
FileUtils.cp_r(File.join(gem_dir, "src"), vendor)
FileUtils.cp_r(File.join(gem_dir, "include"), vendor)
FileUtils.cp(File.join(gem_dir, "LICENSE.md"), vendor)
version = File.read(File.join(gem_dir, "include", "prism", "version.h"))[/PRISM_VERSION "([^"]+)"/, 1]
File.write(File.join(vendor, "VERSION"), "#{version}\n")

# The C extension is replaced by src/vm/stdlib/prism_backend.rb, and the FFI
# backend is not used.
lib = File.join(gem_dir, "lib")
skipped = %w[prism/ffi.rb]
sources = Dir.glob("**/*.rb", base: lib).sort.reject { |path| skipped.include?(path) }

library = File.join(root, "src", "vm", "stdlib", "prism")
FileUtils.rm_rf(library)
sources.each do |path|
  text = File.read(File.join(lib, path))
  # An embedded library is required by feature name, so a relative require
  # names the feature it resolves to.
  text = text.gsub(/require_relative "([^"]+)"/) do
    feature = File.expand_path($1, File.join("/", File.dirname(path))).delete_prefix("/")
    %(require "#{feature}")
  end
  destination = File.join(library, path)
  FileUtils.mkdir_p(File.dirname(destination))
  File.write(destination, text)
end

entries = sources.map do |path|
  feature = path.delete_suffix(".rb")
  %(    ("#{feature}", include_str!("prism/#{path}")),\n)
end
File.write(File.join(root, "src", "vm", "stdlib", "prism_libraries.rs"), <<~RUST)
  // Written by scripts/vendor_prism.rb from prism #{version}.

  /// The files of prism's Ruby library, by the feature name each is required as.
  pub(crate) const PRISM_LIBRARIES: &[(&str, &str)] = &[
  #{entries.join.chomp}
  ];
RUST

puts "prism #{version}: #{Dir.glob("**/*.c", base: vendor).size} C files, #{sources.size} Ruby files"
