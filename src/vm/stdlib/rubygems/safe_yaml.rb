# YAML read from a gem server, loaded without building arbitrary objects.

require 'yaml'

module Gem
  module SafeYAML
    PERMITTED_CLASSES = %w[
      Symbol
      Time
      Date
      Gem::Dependency
      Gem::Platform
      Gem::Requirement
      Gem::Specification
      Gem::Version
      Gem::Version::Requirement
    ].freeze

    PERMITTED_SYMBOLS = %w[
      development
      runtime
    ].freeze

    def self.safe_load(input)
      ::Psych.safe_load(input, permitted_classes: PERMITTED_CLASSES, permitted_symbols: PERMITTED_SYMBOLS, aliases: true)
    end

    def self.load(input)
      ::Psych.safe_load(input, permitted_classes: [::Symbol])
    end
  end
end
