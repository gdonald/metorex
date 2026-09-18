# A range written where a condition goes is not a range. It turns on when its
# first side holds and stays on until its second side does.
held = []
10.times { |i| held << i if (i == 4)..(i == 6) }
p held

# Written with three dots, it waits for the next turn before reading its
# second side, so a single turn cannot close it.
held = []
10.times { |i| held << i if (i == 4)...(i == 4) }
p held

# Written with two dots, it reads the second side the moment the first one
# holds, so a turn where both hold is the only one it is on for.
held = []
10.times { |i| held << i if (i == 4)..(i == 4) }
p held

# Each one keeps its own state, and two of them can be joined.
held = []
10.times { |i| held << i if (i == 1)...(i == 2) or (i == 7)...(i == 8) }
p held
