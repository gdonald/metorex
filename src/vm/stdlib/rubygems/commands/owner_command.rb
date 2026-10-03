# `gem owner`: lists, adds, and removes the people who can push a gem.

require 'rubygems/command'
require 'rubygems/gemcutter_utilities'
require 'rubygems/safe_yaml'
require 'rubygems/text'

module Gem
  module Commands
    class OwnerCommand < Gem::Command
      include Gem::Text
      include Gem::GemcutterUtilities

      def initialize
        super "owner", "Manage gem owners of a gem on the push server"
      end

      # Says each owner the server lists for the gem, by email when it has
      # one, cleaned of control characters before the YAML is read.
      def show_owners(name)
        response = rubygems_api_request :get, "api/v1/gems/#{name}/owners.yaml"

        with_response response do |resp|
          owners = Gem::SafeYAML.load clean_text(resp.body)

          say "Owners for gem: #{name}"
          owners.each do |owner|
            say "- #{owner["email"] || owner["handle"] || owner["id"]}"
          end
        end
      end
    end
  end
end
