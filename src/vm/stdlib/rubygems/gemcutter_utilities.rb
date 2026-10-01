# What a command that talks to a gem server does with the answer.

require 'net/http'
require 'rubygems/text'

module Gem
  # RubyGems names the HTTP classes under its own namespace.
  Net = ::Net unless const_defined?(:Net, false)

  # The command that includes this is the one that says things.
  module GemcutterUtilities
    include Gem::Text

    ERROR_CODE = 1

    # Hands a successful answer to the block, or says its body. Anything
    # else is said, cleaned of control characters, and ends the command.
    def with_response(response, error_prefix = nil)
      case response
      when Gem::Net::HTTPSuccess then
        if block_given?
          yield response
        else
          say clean_text(response.body)
        end
      when Gem::Net::HTTPPermanentRedirect, Gem::Net::HTTPRedirection then
        message = "The request has redirected permanently to #{response["location"]}. Please check your defined push host URL."
        message = "#{error_prefix}: #{message}" if error_prefix

        say clean_text(message)
        terminate_interaction(ERROR_CODE)
      else
        message = response.body
        message = "#{error_prefix}: #{message}" if error_prefix

        say clean_text(message)
        terminate_interaction(ERROR_CODE)
      end
    end
  end
end
