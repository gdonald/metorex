#include "ruby.h"
#include "ruby/debug.h"

static VALUE describe(const rb_debug_inspector_t *inspector, void *data) {
  VALUE locations = rb_debug_inspector_backtrace_locations(inspector);
  VALUE frames = rb_ary_new();
  long index;
  for (index = 0; index < RARRAY_LEN(locations); index++) {
    VALUE frame = rb_ary_new();
    rb_ary_push(frame, rb_debug_inspector_frame_self_get(inspector, index));
    rb_ary_push(frame, rb_debug_inspector_frame_class_get(inspector, index));
    rb_ary_push(frame, rb_debug_inspector_frame_binding_get(inspector, index));
    rb_ary_push(frame, rb_debug_inspector_frame_iseq_get(inspector, index));
    rb_ary_push(frame, rb_ary_entry(locations, index));
    rb_ary_push(frames, frame);
  }
  return frames;
}

static VALUE past_the_end(const rb_debug_inspector_t *inspector, void *data) {
  return rb_debug_inspector_frame_self_get(inspector, (long)data);
}

static VALUE frames(VALUE self) { return rb_debug_inspector_open(describe, NULL); }
static VALUE frame_at(VALUE self, VALUE index) {
  return rb_debug_inspector_open(past_the_end, (void *)NUM2LONG(index));
}

void Init_c_debug(void) {
  VALUE cls = rb_define_class("CDebug", rb_cObject);
  rb_define_method(cls, "frames", frames, 0);
  rb_define_method(cls, "frame_at", frame_at, 1);
  rb_define_const(cls, "LOADED_FROM", rb_debug_inspector_open(describe, NULL));
}
