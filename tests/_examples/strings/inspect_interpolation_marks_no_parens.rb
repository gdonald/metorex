# `inspect` escapes a `#` that would start an interpolation, whatever the
# string's encoding, so the answer reads back as the same text.
p "\#{total}".b
p "\#$stdout".b
p "\#@count".b
p "#plain".b
p "\#{total}".force_encoding("EUC-JP")
p "\#{\xA4\xA2".force_encoding("EUC-JP")
p "\#$stdout".force_encoding("Shift_JIS")
p "\#{total}"
