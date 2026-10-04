#ifndef RUBY_IO_H
#define RUBY_IO_H 1

#include "ruby/ruby.h"
#include "ruby/encoding.h"
#include <sys/select.h>

#ifdef __cplusplus
extern "C" {
#endif

#define FMODE_READABLE 0x00000001
#define FMODE_WRITABLE 0x00000002
#define FMODE_READWRITE (FMODE_READABLE | FMODE_WRITABLE)
#define FMODE_BINMODE 0x00000004
#define FMODE_SYNC 0x00000008
#define FMODE_TTY 0x00000010
#define FMODE_DUPLEX 0x00000020
#define FMODE_APPEND 0x00000040
#define FMODE_CREATE 0x00000080
#define FMODE_EXCL 0x00000400
#define FMODE_TRUNC 0x00000800
#define FMODE_TEXTMODE 0x00001000

#define RUBY_IO_READABLE 1
#define RUBY_IO_PRIORITY 2
#define RUBY_IO_WRITABLE 4

struct rb_io_encoding {
  rb_encoding *enc;
  rb_encoding *enc2;
  int ecflags;
  VALUE ecopts;
};

/* metorex keeps one of these for each IO C asks about, filled from the IO
 * when C asks for it. */
typedef struct rb_io {
  VALUE self;
  int fd;
  int mode;
  VALUE pathv;
} rb_io_t;

rb_io_t *rb_metorex_io_struct(VALUE io);
VALUE rb_io_taint_check(VALUE io);
void rb_io_check_closed(rb_io_t *fptr);
void rb_io_check_readable(rb_io_t *fptr);
void rb_io_check_writable(rb_io_t *fptr);
void rb_io_set_nonblock(rb_io_t *fptr);
#define RB_IO_POINTER(obj, fp) rb_io_check_closed((fp) = rb_metorex_io_struct(rb_io_taint_check(obj)))
#define GetOpenFile RB_IO_POINTER

int rb_io_descriptor(VALUE io);
int rb_io_mode(VALUE io);
VALUE rb_io_path(VALUE io);
VALUE rb_io_closed_p(VALUE io);
VALUE rb_io_check_io(VALUE io);
VALUE rb_io_addstr(VALUE io, VALUE string);
VALUE rb_io_printf(int count, const VALUE *values, VALUE io);
VALUE rb_io_print(int count, const VALUE *values, VALUE io);
VALUE rb_io_puts(int count, const VALUE *values, VALUE io);
VALUE rb_io_write(VALUE io, VALUE string);
VALUE rb_io_close(VALUE io);
VALUE rb_io_binmode(VALUE io);
VALUE rb_io_wait(VALUE io, VALUE events, VALUE timeout);
VALUE rb_io_maybe_wait(int error, VALUE io, VALUE events, VALUE timeout);
int rb_io_maybe_wait_readable(int error, VALUE io, VALUE timeout);
int rb_io_maybe_wait_writable(int error, VALUE io, VALUE timeout);
int rb_thread_wait_fd(int fd);
int rb_thread_fd_writable(int fd);
void rb_fd_fix_cloexec(int fd);
int rb_cloexec_open(const char *path, int flags, mode_t mode);
int rb_cloexec_dup(int fd);
int rb_cloexec_fcntl_dupfd(int fd, int minimum);
VALUE rb_io_open_descriptor(VALUE klass, int descriptor, int mode, VALUE path, VALUE timeout,
                            struct rb_io_encoding *encoding);

typedef struct {
  int maxfd;
  fd_set *fdset;
} rb_fdset_t;
static inline void rb_fd_init(rb_fdset_t *set) {
  set->maxfd = 0;
  set->fdset = (fd_set *)calloc(1, sizeof(fd_set));
}
static inline void rb_fd_set(int fd, rb_fdset_t *set) {
  FD_SET(fd, set->fdset);
  if (fd >= set->maxfd) set->maxfd = fd + 1;
}
static inline void rb_fd_term(rb_fdset_t *set) {
  free(set->fdset);
  set->fdset = NULL;
  set->maxfd = 0;
}
int rb_thread_fd_select(int count, rb_fdset_t *read, rb_fdset_t *write, rb_fdset_t *except,
                        struct timeval *timeout);

#ifdef __cplusplus
}
#endif

#endif
