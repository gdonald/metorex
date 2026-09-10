# encoding: binary
# A source written in bytes keeps each numeric escape as its own byte rather
# than spelling a character out of a run of them.
p("\343\203\255".length)
p("\xE3\x83\xAD".length)
p(%[\xE3\x83\xAD].length)
