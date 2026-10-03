#include "ruby.h"
#include "ruby/thread.h"
#include <errno.h>
#include <pthread.h>
#include <unistd.h>

static VALUE alone(VALUE self) { return rb_thread_alone() ? Qtrue : Qfalse; }
static VALUE current(VALUE self) { return rb_thread_current(); }
static VALUE local_get(VALUE self, VALUE thread, VALUE name) { return rb_thread_local_aref(thread, SYM2ID(name)); }
static VALUE local_set(VALUE self, VALUE thread, VALUE name, VALUE value) {
  return rb_thread_local_aset(thread, SYM2ID(name), value);
}
static VALUE wakeup(VALUE self, VALUE thread) { return rb_thread_wakeup(thread); }

static VALUE wait_for(VALUE self, VALUE microseconds) {
  struct timeval interval;
  interval.tv_sec = 0;
  interval.tv_usec = NUM2INT(microseconds);
  rb_thread_wait_for(interval);
  return Qnil;
}

static VALUE run_proc(void *data) {
  VALUE pair = (VALUE)data;
  VALUE argument = rb_ary_pop(pair);
  VALUE procedure = rb_ary_pop(pair);
  return rb_funcall(procedure, rb_intern("call"), 1, argument);
}

static VALUE create(VALUE self, VALUE procedure, VALUE argument) {
  VALUE pair = rb_ary_new();
  rb_ary_push(pair, procedure);
  rb_ary_push(pair, argument);
  return rb_thread_create(run_proc, (void *)pair);
}

static VALUE native_here(VALUE self) {
  return rb_ary_new_from_args(2, ruby_native_thread_p() ? Qtrue : Qfalse, ruby_thread_has_gvl_p() ? Qtrue : Qfalse);
}

static int checked_elsewhere;
static void *check_elsewhere(void *unused) {
  checked_elsewhere = ruby_native_thread_p();
  return NULL;
}
static VALUE native_elsewhere(VALUE self) {
  pthread_t other;
  pthread_create(&other, NULL, check_elsewhere, NULL);
  pthread_join(other, NULL);
  return checked_elsewhere ? Qtrue : Qfalse;
}

static void *read_one(void *data) {
  int descriptor = *(int *)data;
  char byte = ' ';
  ssize_t count;
  do {
    count = read(descriptor, &byte, 1);
  } while (count == -1 && errno == EINTR);
  close(descriptor);
  return (void *)(count == 1 && byte == 'A' ? Qtrue : Qfalse);
}

static void write_one(void *data) {
  int descriptor = *(int *)data;
  char byte = 'A';
  ssize_t count;
  do {
    count = write(descriptor, &byte, 1);
  } while (count == -1 && errno == EINTR);
  close(descriptor);
}

static void *add_numbers(void *data) {
  long *numbers = data;
  numbers[2] = numbers[0] + numbers[1];
  return (void *)&numbers[2];
}

static VALUE blocking_read(VALUE self) {
  int descriptors[2];
  if (pipe(descriptors) == -1) rb_raise(rb_eRuntimeError, "could not make a pipe");
  return (VALUE)rb_thread_call_without_gvl(read_one, &descriptors[0], write_one, &descriptors[1]);
}

static void *read_until_interrupted(void *data) {
  int descriptor = (int)(size_t)data;
  char byte;
  return (void *)(read(descriptor, &byte, 1) == -1 && errno == EINTR ? Qtrue : Qfalse);
}

static VALUE interrupted_read(VALUE self) {
  int descriptors[2];
  if (pipe(descriptors) == -1) rb_raise(rb_eRuntimeError, "could not make a pipe");
  void *answer = rb_thread_call_without_gvl(read_until_interrupted, (void *)(size_t)descriptors[0], RUBY_UBF_IO, 0);
  close(descriptors[0]);
  close(descriptors[1]);
  return (VALUE)answer;
}

static VALUE sum_without_lock(VALUE self, VALUE first, VALUE second) {
  long numbers[3] = {NUM2LONG(first), NUM2LONG(second), 0};
  long *sum = rb_thread_call_without_gvl(add_numbers, numbers, NULL, NULL);
  return LONG2NUM(*sum);
}

void Init_c_threads(void) {
  VALUE klass = rb_define_class("CThreads", rb_cObject);
  rb_define_method(klass, "alone", alone, 0);
  rb_define_method(klass, "current", current, 0);
  rb_define_method(klass, "local_get", local_get, 2);
  rb_define_method(klass, "local_set", local_set, 3);
  rb_define_method(klass, "wakeup", wakeup, 1);
  rb_define_method(klass, "wait_for", wait_for, 1);
  rb_define_method(klass, "create", create, 2);
  rb_define_method(klass, "native_here", native_here, 0);
  rb_define_method(klass, "native_elsewhere", native_elsewhere, 0);
  rb_define_method(klass, "blocking_read", blocking_read, 0);
  rb_define_method(klass, "interrupted_read", interrupted_read, 0);
  rb_define_method(klass, "sum_without_lock", sum_without_lock, 2);
}
