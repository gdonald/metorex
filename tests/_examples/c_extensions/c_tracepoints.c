#include "ruby.h"
#include "ruby/debug.h"

static void record(VALUE trace, void *data) {
  rb_funcall((VALUE)data, rb_intern("push"), 1, rb_funcall(trace, rb_intern("event"), 0));
}

static VALUE trace_new(VALUE self, VALUE events, VALUE seen) {
  return rb_tracepoint_new(Qnil, (rb_event_flag_t)NUM2UINT(events), record, (void *)seen);
}
static VALUE enable(VALUE self, VALUE trace) {
  rb_tracepoint_enable(trace);
  return rb_tracepoint_enabled_p(trace);
}
static VALUE disable(VALUE self, VALUE trace) {
  rb_tracepoint_disable(trace);
  return rb_tracepoint_enabled_p(trace);
}

void Init_c_tracepoints(void) {
  VALUE cls = rb_define_class("CTracepoints", rb_cObject);
  rb_define_const(cls, "LINE", UINT2NUM(RUBY_EVENT_LINE));
  rb_define_const(cls, "CALL", UINT2NUM(RUBY_EVENT_CALL));
  rb_define_const(cls, "RETURN", UINT2NUM(RUBY_EVENT_RETURN));
  rb_define_const(cls, "NONE", UINT2NUM(RUBY_EVENT_NONE));
  rb_define_method(cls, "trace_new", trace_new, 2);
  rb_define_method(cls, "enable", enable, 1);
  rb_define_method(cls, "disable", disable, 1);
}
