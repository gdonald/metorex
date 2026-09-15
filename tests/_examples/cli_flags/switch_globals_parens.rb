# Under `-s`, each switch written among the program's own arguments binds a
# global of the same name, with the dashes in it turned into underscores.
p($flag)
p($name)
p($_long__name)
p(ARGV)
