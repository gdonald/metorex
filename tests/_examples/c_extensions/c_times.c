#include "ruby.h"

static VALUE pair(long first, long second) {
  VALUE values = rb_ary_new();
  rb_ary_push(values, LONG2NUM(first));
  rb_ary_push(values, LONG2NUM(second));
  return values;
}

static VALUE from_micro(VALUE self, VALUE seconds, VALUE micro) {
  return rb_time_new(NUM2TIMET(seconds), NUM2LONG(micro));
}
static VALUE from_nano(VALUE self, VALUE seconds, VALUE nano) {
  return rb_time_nano_new(NUM2TIMET(seconds), NUM2LONG(nano));
}
static VALUE from_number(VALUE self, VALUE seconds, VALUE offset) { return rb_time_num_new(seconds, offset); }
static VALUE from_timespec(VALUE self, VALUE seconds, VALUE nano, VALUE offset) {
  struct timespec time;
  time.tv_sec = NUM2TIMET(seconds);
  time.tv_nsec = NUM2LONG(nano);
  return rb_time_timespec_new(&time, (int)NUM2LONG(offset));
}
static VALUE now(VALUE self) {
  struct timespec time;
  rb_timespec_now(&time);
  return rb_time_timespec_new(&time, 0x7ffffffe);
}
static VALUE interval(VALUE self, VALUE seconds) {
  struct timeval held = rb_time_interval(seconds);
  return pair(held.tv_sec, held.tv_usec);
}
static VALUE timeval(VALUE self, VALUE time) {
  struct timeval held = rb_time_timeval(time);
  return pair(held.tv_sec, held.tv_usec);
}
static VALUE timespec(VALUE self, VALUE time) {
  struct timespec held = rb_time_timespec(time);
  return pair(held.tv_sec, held.tv_nsec);
}
static VALUE push_onto(VALUE self, VALUE array, VALUE element) { return rb_ary_push(array, element); }

void Init_c_times(void) {
  VALUE cls = rb_define_class("CTimes", rb_cObject);
  rb_define_method(cls, "from_micro", from_micro, 2);
  rb_define_method(cls, "from_nano", from_nano, 2);
  rb_define_method(cls, "from_number", from_number, 2);
  rb_define_method(cls, "from_timespec", from_timespec, 3);
  rb_define_method(cls, "now", now, 0);
  rb_define_method(cls, "interval", interval, 1);
  rb_define_method(cls, "timeval", timeval, 1);
  rb_define_method(cls, "timespec", timespec, 1);
  rb_define_method(cls, "push_onto", push_onto, 2);
}
