#include "ruby.h"
#include "ruby/io.h"
#include <errno.h>
#include <fcntl.h>
#include <unistd.h>

static VALUE descriptor(VALUE self, VALUE io) { return INT2FIX(rb_io_descriptor(io)); }
static VALUE mode(VALUE self, VALUE io) { return INT2FIX(rb_io_mode(io)); }
static VALUE struct_fields(VALUE self, VALUE io) {
  rb_io_t *fptr = rb_metorex_io_struct(io);
  return rb_ary_new_from_args(3, INT2FIX(fptr->fd), INT2FIX(fptr->mode), fptr->pathv);
}
static VALUE check_readable(VALUE self, VALUE io) {
  rb_io_t *fptr;
  GetOpenFile(io, fptr);
  rb_io_check_readable(fptr);
  return Qtrue;
}
static VALUE check_writable(VALUE self, VALUE io) {
  rb_io_t *fptr;
  GetOpenFile(io, fptr);
  rb_io_check_writable(fptr);
  return Qtrue;
}
static VALUE set_nonblock(VALUE self, VALUE io) {
  rb_io_t *fptr;
  GetOpenFile(io, fptr);
  rb_io_set_nonblock(fptr);
  return (fcntl(fptr->fd, F_GETFL) & O_NONBLOCK) ? Qtrue : Qfalse;
}
static VALUE fix_cloexec(VALUE self, VALUE io) {
  rb_fd_fix_cloexec(rb_io_descriptor(io));
  return (fcntl(rb_io_descriptor(io), F_GETFD) & FD_CLOEXEC) ? Qtrue : Qfalse;
}
static VALUE cloexec_open(VALUE self, VALUE path) {
  int fd = rb_cloexec_open(StringValueCStr(path), O_RDONLY, 0);
  VALUE answer = rb_ary_new_from_args(2, (fcntl(fd, F_GETFD) & FD_CLOEXEC) ? Qtrue : Qfalse, fd > 2 ? Qtrue : Qfalse);
  close(fd);
  return answer;
}
static VALUE cloexec_dup(VALUE self, VALUE io, VALUE minimum) {
  int fd = NIL_P(minimum) ? rb_cloexec_dup(rb_io_descriptor(io))
                          : rb_cloexec_fcntl_dupfd(rb_io_descriptor(io), FIX2INT(minimum));
  VALUE answer = rb_ary_new_from_args(2, (fcntl(fd, F_GETFD) & FD_CLOEXEC) ? Qtrue : Qfalse, INT2FIX(fd));
  close(fd);
  return answer;
}
static VALUE path(VALUE self, VALUE io) { return rb_io_path(io); }
static VALUE closed(VALUE self, VALUE io) { return rb_io_closed_p(io); }
static VALUE check_io(VALUE self, VALUE object) { return rb_io_check_io(object); }
static VALUE taint_check(VALUE self, VALUE io) { return rb_io_taint_check(io); }
static VALUE addstr(VALUE self, VALUE io, VALUE text) { return rb_io_addstr(io, text); }
static VALUE io_printf(VALUE self, VALUE io, VALUE values) {
  return rb_io_printf(RARRAY_LENINT(values), RARRAY_PTR(values), io);
}
static VALUE io_print(VALUE self, VALUE io, VALUE values) { return rb_io_print(RARRAY_LENINT(values), RARRAY_PTR(values), io); }
static VALUE io_puts(VALUE self, VALUE io, VALUE values) { return rb_io_puts(RARRAY_LENINT(values), RARRAY_PTR(values), io); }
static VALUE io_write(VALUE self, VALUE io, VALUE text) { return rb_io_write(io, text); }
static VALUE io_close(VALUE self, VALUE io) { return rb_io_close(io); }
static VALUE binmode(VALUE self, VALUE io) { return rb_io_binmode(io); }
static VALUE io_wait(VALUE self, VALUE io, VALUE events, VALUE timeout) { return rb_io_wait(io, events, timeout); }
static VALUE maybe_wait(VALUE self, VALUE error, VALUE io, VALUE events, VALUE timeout) {
  return rb_io_maybe_wait(NUM2INT(error), io, events, timeout);
}
static VALUE maybe_wait_readable(VALUE self, VALUE error, VALUE io, VALUE timeout) {
  return INT2FIX(rb_io_maybe_wait_readable(NUM2INT(error), io, timeout));
}
static VALUE maybe_wait_writable(VALUE self, VALUE error, VALUE io, VALUE timeout) {
  return INT2FIX(rb_io_maybe_wait_writable(NUM2INT(error), io, timeout));
}
static VALUE wait_fd(VALUE self, VALUE io) { return INT2FIX(rb_thread_wait_fd(rb_io_descriptor(io))); }
static VALUE fd_writable(VALUE self, VALUE io) { return INT2FIX(rb_thread_fd_writable(rb_io_descriptor(io))); }
static VALUE fd_select(VALUE self, VALUE readers, VALUE writers, VALUE priority, VALUE seconds) {
  rb_fdset_t read, write, except;
  int max = 0;
  long index;
  rb_fd_init(&read);
  rb_fd_init(&write);
  rb_fd_init(&except);
  for (index = 0; index < RARRAY_LEN(readers); index++) rb_fd_set(rb_io_descriptor(RARRAY_AREF(readers, index)), &read);
  for (index = 0; index < RARRAY_LEN(writers); index++) rb_fd_set(rb_io_descriptor(RARRAY_AREF(writers, index)), &write);
  for (index = 0; index < RARRAY_LEN(priority); index++) rb_fd_set(rb_io_descriptor(RARRAY_AREF(priority, index)), &except);
  max = read.maxfd > write.maxfd ? read.maxfd : write.maxfd;
  if (except.maxfd > max) max = except.maxfd;
  struct timeval timeout = {NIL_P(seconds) ? 0 : FIX2INT(seconds), 0};
  int ready = rb_thread_fd_select(max, RARRAY_LEN(readers) ? &read : NULL, RARRAY_LEN(writers) ? &write : NULL,
                                  &except, NIL_P(seconds) ? NULL : &timeout);
  VALUE marked = rb_ary_new();
  for (index = 0; index < RARRAY_LEN(readers); index++)
    rb_ary_push(marked, FD_ISSET(rb_io_descriptor(RARRAY_AREF(readers, index)), read.fdset) ? Qtrue : Qfalse);
  rb_fd_term(&read);
  rb_fd_term(&write);
  rb_fd_term(&except);
  return rb_ary_new_from_args(2, INT2FIX(ready), marked);
}
static VALUE open_descriptor(VALUE self, VALUE klass, VALUE fd, VALUE fmode, VALUE path, VALUE timeout, VALUE internal,
                             VALUE external) {
  if (NIL_P(internal)) return rb_io_open_descriptor(klass, FIX2INT(fd), FIX2INT(fmode), path, timeout, NULL);
  struct rb_io_encoding encoding;
  encoding.enc = rb_to_encoding(internal);
  encoding.enc2 = rb_to_encoding(external);
  encoding.ecflags = 0;
  encoding.ecopts = Qnil;
  return rb_io_open_descriptor(klass, FIX2INT(fd), FIX2INT(fmode), path, timeout, &encoding);
}

void Init_c_io(void) {
  VALUE cls = rb_define_class("CIO", rb_cObject);
  rb_define_method(cls, "descriptor", descriptor, 1);
  rb_define_method(cls, "mode", mode, 1);
  rb_define_method(cls, "struct_fields", struct_fields, 1);
  rb_define_method(cls, "check_readable", check_readable, 1);
  rb_define_method(cls, "check_writable", check_writable, 1);
  rb_define_method(cls, "set_nonblock", set_nonblock, 1);
  rb_define_method(cls, "fix_cloexec", fix_cloexec, 1);
  rb_define_method(cls, "cloexec_open", cloexec_open, 1);
  rb_define_method(cls, "cloexec_dup", cloexec_dup, 2);
  rb_define_method(cls, "path", path, 1);
  rb_define_method(cls, "closed", closed, 1);
  rb_define_method(cls, "check_io", check_io, 1);
  rb_define_method(cls, "taint_check", taint_check, 1);
  rb_define_method(cls, "addstr", addstr, 2);
  rb_define_method(cls, "io_printf", io_printf, 2);
  rb_define_method(cls, "io_print", io_print, 2);
  rb_define_method(cls, "io_puts", io_puts, 2);
  rb_define_method(cls, "io_write", io_write, 2);
  rb_define_method(cls, "io_close", io_close, 1);
  rb_define_method(cls, "binmode", binmode, 1);
  rb_define_method(cls, "io_wait", io_wait, 3);
  rb_define_method(cls, "maybe_wait", maybe_wait, 4);
  rb_define_method(cls, "maybe_wait_readable", maybe_wait_readable, 3);
  rb_define_method(cls, "maybe_wait_writable", maybe_wait_writable, 3);
  rb_define_method(cls, "wait_fd", wait_fd, 1);
  rb_define_method(cls, "fd_writable", fd_writable, 1);
  rb_define_method(cls, "fd_select", fd_select, 4);
  rb_define_method(cls, "open_descriptor", open_descriptor, 7);
}
