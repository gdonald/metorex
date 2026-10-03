# A NoMethodError for an instance of a class with no name names the class
# the way the class inspects, whether the method is missing or private.
klass = Class.new do
  private

  def hidden; end
end

[:hidden, :missing].each do |name|
  begin
    klass.new.public_send name
  rescue NoMethodError => error
    puts error.message.sub(/0x\h+/, "0xADDRESS")
  end
end
