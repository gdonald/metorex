pub(super) const SOURCE: &str = r##"
module Process
  # The same ids Process itself answers, gathered under the words Ruby
  # gathers them under.
  module GID
    def self.rid
      Process.gid
    end

    def self.eid
      Process.egid
    end

    def self.eid= wanted
      Process.egid = wanted
    end

    def self.change_privilege wanted
      Process.gid = wanted
      wanted
    end
  end

  module UID
    def self.rid
      Process.uid
    end

    def self.eid
      Process.euid
    end

    def self.eid= wanted
      Process.euid = wanted
    end

    def self.change_privilege wanted
      Process.uid = wanted
      wanted
    end
  end

  module Sys
    def self.getgid
      Process.gid
    end

    def self.getuid
      Process.uid
    end

    def self.getegid
      Process.egid
    end

    def self.geteuid
      Process.euid
    end

    def self.setgid wanted
      Process.gid = wanted
    end

    def self.setuid wanted
      Process.uid = wanted
    end

    def self.setegid wanted
      Process.egid = wanted
    end

    def self.seteuid wanted
      Process.euid = wanted
    end
  end
end"##;
