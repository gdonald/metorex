# Writes src/vm/native_methods/core_method_names.rs: the instance methods each
# core class and module defines in MRI, by visibility. Run it with MRI's core
# alone loaded:
#
#   ruby --disable=gems,did_you_mean,error_highlight,syntax_suggest scripts/generate_core_method_names.rb
output = File.expand_path("../src/vm/native_methods/core_method_names.rs", __dir__)

seen = {}
walk = lambda do |mod|
  next if seen[mod]

  seen[mod] = true
  mod.constants(false).each do |name|
    value = begin
      mod.const_get(name)
    rescue NameError
      next
    end
    prefix = mod == Object ? "" : "#{mod.name}::"
    walk.call(value) if value.is_a?(Module) && value.name&.start_with?(prefix)
  end
end
walk.call(Object)

# Classes metorex defines from MRI's own Ruby source rather than natively,
# whose methods are listed from that source.
FROM_RUBY_SOURCE = %w[Pathname].freeze

listed = ->(names) { "&[#{names.map(&:to_s).sort.map(&:inspect).join(", ")}]" }
rows = seen.keys.select(&:name).reject { |mod| FROM_RUBY_SOURCE.include?(mod.name) }.sort_by(&:name).map do |mod|
  public_names = mod.public_instance_methods(false)
  private_names = mod.private_instance_methods(false)
  protected_names = mod.protected_instance_methods(false)
  next if public_names.empty? && private_names.empty? && protected_names.empty?

  "    CoreMethods {\n        owner: #{mod.name.inspect},\n        public: #{listed.(public_names)},\n" \
    "        private: #{listed.(private_names)},\n        protected: #{listed.(protected_names)},\n    },\n"
end.compact
# The object a program runs against at the top level answers methods of its
# own singleton class, which a listing of it reads under the name `main`.
main = TOPLEVEL_BINDING.receiver.singleton_class
rows.unshift("    CoreMethods {\n        owner: \"main\",\n        public: #{listed.(main.public_instance_methods(false))},\n" \
  "        private: #{listed.(main.private_instance_methods(false))},\n        protected: #{listed.(main.protected_instance_methods(false))},\n    },\n")

File.write(output, <<~RUST)
  // Written by scripts/generate_core_method_names.rb from Ruby #{RUBY_VERSION}.

  /// The instance methods a core class or module defines in Ruby, by
  /// visibility, which the method listings of a built-in value draw from.
  pub(crate) struct CoreMethods {
      pub(crate) owner: &'static str,
      pub(crate) public: &'static [&'static str],
      pub(crate) private: &'static [&'static str],
      pub(crate) protected: &'static [&'static str],
  }

  pub(crate) const CORE_METHODS: &[CoreMethods] = &[
  #{rows.join.chomp}
  ];
RUST
puts "#{rows.size} classes and modules written to #{output}"
