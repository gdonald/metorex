# frozen_string_literal: true

require "irb/command/debug"

module IRB
  # :stopdoc:

  module Command
    class Delete < DebugCommand
      def execute(arg)
        execute_debug_command(pre_cmds: "delete #{arg}")
      end
    end
  end

  # :startdoc:
end
