# `gem owner` says each owner a server lists, by email, handle, or id, with
# control characters cleaned out of the answer before its YAML is read.
require "rubygems"
require "rubygems/commands/owner_command"

class Recording < Gem::StreamUI
  attr_reader :said

  def initialize
    super nil, nil, nil, false
    @said = []
  end

  def say statement = ""
    @said << statement
  end
end

command = Gem::Commands::OwnerCommand.new
p [command.command, command.summary, command.options]

def command.rubygems_api_request *args
  response = Gem::Net::HTTPOK.new nil, nil, nil
  def response.body
    "---\n- email: \"\e]2;title\a\"\n- handle: maintainer\n- id: 7\n"
  end
  response
end

recording = Recording.new
command.ui = recording
command.show_owners "rake"
p recording.said

p Gem::SafeYAML.safe_load "---\n- runtime\n"
Gem::DefaultUserInteraction.ui = nil
