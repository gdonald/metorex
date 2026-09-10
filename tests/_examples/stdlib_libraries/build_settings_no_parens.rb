require 'rbconfig'
require 'rbconfig/sizeof'
require 'resolv'
require 'optparse'

# What the interpreter was built as, all of it named with strings.
p RbConfig::CONFIG.values_at("MAJOR", "MINOR", "TEENY") == RUBY_VERSION.split(".")
p RbConfig::CONFIG["PATCHLEVEL"] == RUBY_PATCHLEVEL.to_s
p RUBY_PLATFORM.include?(RbConfig::CONFIG["host_cpu"])
p RUBY_DESCRIPTION.include?(RUBY_VERSION)
p RbConfig::TOPDIR
p RbConfig::SIZEOF["void*"] * 8
p [RbConfig::SIZEOF["float"], RbConfig::SIZEOF["double"]]
p RbConfig::LIMITS["FIXNUM_MAX"] > 0
p [RbConfig::LIMITS["SHRT_MIN"], RbConfig::LIMITS["SHRT_MAX"]]

# A name and an address, read out of a hosts file.
hosts = Resolv::Hosts.new("/etc/hosts")
resolver = Resolv.new([hosts])
p resolver.getaddress("localhost")
p resolver.getnames("127.0.0.1").include?("localhost")
begin
  Resolv.new([]).getaddress("nothing.answers.to.this.")
rescue Resolv::ResolvError => problem
  p(problem.class)
end

# Options read against a description of what they mean.
settings = {}
parser = OptionParser.new do |options|
  options.on("-v", "--[no-]verbose", "Run verbosely")
  options.on("-r", "--require LIBRARY", "Require LIBRARY first")
end
rest = parser.parse(%w[--verbose --require optparse leftover], into: settings)
p settings
p rest
p parser.parse(%w[--no-verbose], into: {}) && settings[:verbose]
