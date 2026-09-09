# Every path under a directory, walked one at a time.
module Find
  # Walks the paths given and hands over each one it reaches, the directories
  # among them included. A directory the block prunes is not descended into.
  def self.find(*paths, ignore_error: true, &block)
    return self.to_enum(:find, *paths, ignore_error: ignore_error) if block.nil?
    paths.each do |top|
      walked = [top.to_s]
      while !walked.empty?
        path = walked.shift
        catch(:prune) do
          block.call(path.dup)
          if File.directory?(path)
            names = Dir.entries(path).reject { |name| name == "." || name == ".." }
            names.sort.reverse.each do |name|
              walked.unshift(path == "/" ? "/" + name : path + "/" + name)
            end
          end
        end
      end
    end
    nil
  end

  # Called from inside a `find` block to leave the directory it was handed
  # unwalked.
  def self.prune
    throw :prune
  end

  def find(*paths, ignore_error: true, &block)
    Find.find(*paths, ignore_error: ignore_error, &block)
  end

  def prune
    Find.prune
  end
end
