# The reading a console offers, as far as a string standing in for one can
# carry it.
class StringIO
  def getch
    getc
  end

  def getpass(prompt = nil)
    self.write(prompt) unless prompt.nil?
    line = self.gets
    line.nil? ? nil : line.gsub(/\n\z/, "")
  end
end
