# The numbers `IO#fcntl` takes, which name what to ask a descriptor about or
# what to set on it.
module Fcntl
  F_DUPFD = 0
  F_GETFD = 1
  F_SETFD = 2
  F_GETFL = 3
  F_SETFL = 4
  F_GETOWN = 5
  F_SETOWN = 6
  F_GETLK = 7
  F_SETLK = 8
  F_SETLKW = 9
  FD_CLOEXEC = 1

  O_RDONLY = File::RDONLY
  O_WRONLY = File::WRONLY
  O_RDWR = File::RDWR
  O_CREAT = File::CREAT
  O_EXCL = File::EXCL
  O_TRUNC = File::TRUNC
  O_APPEND = File::APPEND
  O_NONBLOCK = File::NONBLOCK
  O_NDELAY = File::NONBLOCK
  O_ACCMODE = 3

  F_RDLCK = 1
  F_WRLCK = 3
  F_UNLCK = 2
end
