# The base every `gem` command is built on: its name, a one-line summary,
# and the options it starts with.

require 'rubygems/user_interaction'

module Gem
  class Command
    include Gem::UserInteraction

    HELP = <<-HELP
RubyGems is a package manager for Ruby.

  Usage:
    gem -h/--help
    gem -v/--version
    gem [global options...] command [arguments...] [options...]

  Global options:
    -C PATH                      run as if gem was started in <PATH>
                                 instead of the current working directory

  Examples:
    gem install rake
    gem list --local
    gem build package.gemspec
    gem push package-0.0.1.gem
    gem help install

  Further help:
    gem help commands            list all 'gem' commands
    gem help examples            show some examples of usage
    gem help gem_dependencies    gem dependencies file guide
    gem help platforms           gem platforms guide
    gem help <COMMAND>           show help on COMMAND
                                   (e.g. 'gem help install')
  Further information:
    https://guides.rubygems.org
    HELP

    attr_reader :command, :summary, :defaults, :options

    def initialize(command, summary = nil, defaults = {})
      @command = command
      @summary = summary
      @defaults = defaults
      @options = defaults.dup
    end
  end

  # Where the commands live.
  module Commands
  end
end
