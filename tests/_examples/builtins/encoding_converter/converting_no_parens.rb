# encoding: binary
# A conversion carries text from one encoding to another, refusing what
# neither side can spell.
converter = Encoding::Converter.new "utf-8", "iso-8859-1"
p converter.convert "plain"

begin
  Encoding::Converter.new("utf-8", "ascii").convert "\u{8765}"
rescue Encoding::UndefinedConversionError => refused
  p refused.source_encoding_name
  p refused.destination_encoding_name
  p refused.error_char
end

begin
  converter.convert "\xf1abcd"
rescue Encoding::InvalidByteSequenceError => refused
  p refused.source_encoding_name
  p refused.destination_encoding_name
  p refused.error_bytes.bytes
  p refused.readagain_bytes.bytes
end

# The trouble belongs to the step it stopped, which for a conversion that
# passes through UTF-8 is not always the pair the converter was built with.
indirect = Encoding::Converter.new "EUC-JP", "ISO-8859-1"
begin
  indirect.convert "abc\xA1\xFFdef"
rescue Encoding::InvalidByteSequenceError => refused
  p refused.destination_encoding_name
end

# An encoding stands for an Encoding rather than for the class it is held as.
p Encoding::UTF_8.class
p "held".encoding.instance_of? Encoding
