#include "ruby.h"

static VALUE described(RB_BLOCK_CALL_FUNC_ARGLIST(first, data)) {
  return rb_ary_new_from_args(5, first, data, INT2FIX(argc), rb_ary_new_from_values(argc, argv),
                              rb_block_given_p() ? Qtrue : Qfalse);
}
static VALUE handed_block(RB_BLOCK_CALL_FUNC_ARGLIST(first, data)) { return blockarg; }

static VALUE make(VALUE self, VALUE data) { return rb_proc_new(described, data); }
static VALUE make_block_reader(VALUE self) { return rb_proc_new(handed_block, Qnil); }
static VALUE arity(VALUE self, VALUE procedure) { return INT2FIX(rb_proc_arity(procedure)); }
static VALUE is_proc(VALUE self, VALUE value) { return rb_obj_is_proc(value); }
static VALUE call(VALUE self, VALUE procedure, VALUE values) { return rb_proc_call(procedure, values); }
static VALUE call_keywords(VALUE self, VALUE procedure, VALUE values) {
  return rb_proc_call_kw(procedure, values, RB_PASS_KEYWORDS);
}
static VALUE call_block(VALUE self, VALUE procedure, VALUE values, VALUE block) {
  return rb_proc_call_with_block(procedure, (int)RARRAY_LEN(values), RARRAY_PTR(values), block);
}
static VALUE call_block_keywords(VALUE self, VALUE procedure, VALUE values, VALUE block) {
  return rb_proc_call_with_block_kw(procedure, (int)RARRAY_LEN(values), RARRAY_PTR(values), block,
                                    RB_PASS_KEYWORDS);
}

void Init_c_procs(void) {
  VALUE cls = rb_define_class("CProcs", rb_cObject);
  rb_define_method(cls, "make", make, 1);
  rb_define_method(cls, "make_block_reader", make_block_reader, 0);
  rb_define_method(cls, "arity", arity, 1);
  rb_define_method(cls, "is_proc", is_proc, 1);
  rb_define_method(cls, "call", call, 2);
  rb_define_method(cls, "call_keywords", call_keywords, 2);
  rb_define_method(cls, "call_block", call_block, 3);
  rb_define_method(cls, "call_block_keywords", call_block_keywords, 3);
}
