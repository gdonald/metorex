# Reads the arguments a `gem` command is run with and hands them to the
# command they name.

require 'rubygems/command'

module Gem
  class CommandManager
    include Gem::Text
    include Gem::UserInteraction

    def initialize
      @commands = {}
    end

    def run(args, build_args = nil)
      process_args(args, build_args)
    rescue StandardError => ex
      if ex.respond_to?(:detailed_message)
        msg = ex.detailed_message(highlight: false).sub(/\A(.*?)(?: \(.+?\))/) { $1 }
      else
        msg = ex.message
      end
      alert_error clean_text("While executing gem ... (#{ex.class})\n    #{msg}")
      ui.backtrace ex

      terminate_interaction(1)
    rescue Interrupt
      alert_error clean_text("Interrupted")
      terminate_interaction(1)
    end

    def process_args(args, build_args = nil)
      if args.empty?
        say Gem::Command::HELP
        terminate_interaction 1
      end

      case args.first
      when "-h", "--help" then
        say Gem::Command::HELP
        terminate_interaction 0
      when "-v", "--version" then
        say Gem::VERSION
        terminate_interaction 0
      when "-C" then
        args.shift
        start_point = args.shift
        if Dir.exist?(start_point)
          Dir.chdir(start_point) { invoke_command(args, build_args) }
        else
          alert_error clean_text("#{start_point} isn't a directory.")
          terminate_interaction 1
        end
      when /^-/ then
        alert_error clean_text("Invalid option: #{args.first}. See 'gem --help'.")
        terminate_interaction 1
      else
        invoke_command(args, build_args)
      end
    end

    def [](command_name)
      command_name = command_name.intern
      return nil if @commands[command_name].nil?
      @commands[command_name] ||= load_and_instantiate(command_name)
    end

    def find_command(cmd_name)
      self[cmd_name]
    end

    private

    def load_and_instantiate(command_name)
      command_name = command_name.to_s
      const_name = command_name.capitalize.gsub(/_(.)/) { $1.upcase } << "Command"

      begin
        begin
          require "rubygems/commands/#{command_name}_command"
        rescue LoadError
          # A plugin may have defined it already.
        end

        Gem::Commands.const_get(const_name).new
      rescue StandardError => e
        alert_error clean_text("Loading command: #{command_name} (#{e.class})\n\t#{e}")
        ui.backtrace e
      end
    end

    def invoke_command(args, build_args)
      cmd_name = args.shift.downcase
      cmd = find_command cmd_name
      terminate_interaction 1 unless cmd
      cmd.invoke_with_build_args args, build_args
    end
  end
end
