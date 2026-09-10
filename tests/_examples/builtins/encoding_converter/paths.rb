# A converter names the two encodings it works between and the steps it takes
# to get from one to the other.
converter = Encoding::Converter.new "us-ascii", "utf-8"
p converter.source_encoding
p converter.destination_encoding
p converter.inspect
p converter.convpath
p converter.replacement

p Encoding::Converter.search_convpath("ascii", "Big5")
p Encoding::Converter.search_convpath("ISO-8859-1", "EUC-JP", crlf_newline: true).last
p Encoding::Converter.asciicompat_encoding("UTF-16BE")
p Encoding::Converter.asciicompat_encoding("UTF-8")
p Encoding::Converter::INVALID_MASK.is_a? Integer

begin
  Encoding::Converter.new "utf-8", "utf-8"
rescue Encoding::ConverterNotFoundError => problem
  p problem.class
end

# A replacement written into the options stands in for what the destination
# cannot spell.
p Encoding::Converter.new("us-ascii", "utf-8", replace: "fubar").replacement

# Keywords may come from any object that reads as a Hash.
class Options
  def to_hash
    { replace: "spelled out" }
  end
end
p Encoding::Converter.new("us-ascii", "utf-8", **Options.new).replacement
