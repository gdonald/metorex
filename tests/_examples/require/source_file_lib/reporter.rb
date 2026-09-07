# The file a method was written in is what __FILE__ names inside it, wherever
# the method is later called from.
module Reporter
  def self.own_file
    __FILE__.split("/").last
  end

  def self.file_from_a_block
    [1].map { __FILE__.split("/").last }.first
  end
end
