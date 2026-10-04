#include "ruby.h"
#include "ruby/vm.h"
#include <errno.h>

static VALUE call_with(VALUE pair) {
  return rb_funcall(rb_ary_entry(pair, 0), rb_intern("call"), 1, rb_ary_entry(pair, 1));
}
static VALUE call_with_exception(VALUE pair, VALUE exception) {
  return rb_funcall(rb_ary_entry(pair, 0), rb_intern("call"), 2, rb_ary_entry(pair, 1), exception);
}
static VALUE yield_it(VALUE value) { return rb_yield(value); }

static VALUE protect(VALUE self, VALUE value) {
  int state = 0;
  VALUE answered = rb_protect(yield_it, value, &state);
  return rb_ary_new_from_args(3, answered, INT2FIX(state), rb_errinfo());
}
static VALUE protect_then_jump(VALUE self, VALUE value) {
  int state = 0;
  VALUE answered = rb_protect(yield_it, value, &state);
  if (state) rb_jump_tag(state);
  return answered;
}
static VALUE jump_without_protect(VALUE self) {
  rb_jump_tag(6);
  return Qnil;
}
static VALUE clear_errinfo(VALUE self) {
  rb_set_errinfo(Qnil);
  return Qnil;
}
static VALUE rescue(VALUE self, VALUE body, VALUE handler) {
  return rb_rescue(call_with, body, NIL_P(handler) ? NULL : call_with_exception, handler);
}
static VALUE rescue2(VALUE self, VALUE body, VALUE handler, VALUE first, VALUE second) {
  return rb_rescue2(call_with, body, call_with_exception, handler, first, second, (VALUE)0);
}
static VALUE ensure(VALUE self, VALUE body, VALUE ensured) {
  return rb_ensure(call_with, body, call_with, ensured);
}
static VALUE yield_tag(RB_BLOCK_CALL_FUNC_ARGLIST(tag, callable)) {
  return rb_funcall(callable, rb_intern("call"), 1, tag);
}
static VALUE catch_named(VALUE self, VALUE name, VALUE callable) {
  return rb_catch(StringValueCStr(name), yield_tag, callable);
}
static VALUE catch_object(VALUE self, VALUE tag, VALUE callable) { return rb_catch_obj(tag, yield_tag, callable); }
static VALUE throw_named(VALUE self, VALUE name, VALUE value) {
  rb_throw(StringValueCStr(name), value);
  return Qnil;
}
static VALUE throw_object(VALUE self, VALUE tag, VALUE value) {
  rb_throw_obj(tag, value);
  return Qnil;
}
static VALUE evaluate(VALUE self, VALUE source) { return rb_eval_string(StringValueCStr(source)); }
static VALUE evaluate_protected(VALUE self, VALUE source) {
  int state = 0;
  VALUE answered = rb_eval_string_protect(StringValueCStr(source), &state);
  rb_set_errinfo(Qnil);
  return rb_ary_new_from_args(2, answered, INT2FIX(state));
}
static VALUE nest(VALUE object, VALUE depth, int recursive) {
  if (recursive) return rb_str_new_cstr("recursive");
  if (FIX2INT(depth) == 0) return rb_str_new_cstr("done");
  return rb_exec_recursive(nest, object, INT2FIX(FIX2INT(depth) - 1));
}
static VALUE recurse(VALUE self, VALUE object, VALUE depth) { return rb_exec_recursive(nest, object, depth); }
static VALUE raise_inside(VALUE object, VALUE message, int recursive) {
  rb_raise(rb_eRuntimeError, "%s", StringValueCStr(message));
  return Qnil;
}
static VALUE recurse_raising(VALUE self, VALUE message) { return rb_exec_recursive(raise_inside, self, message); }
static VALUE need_block(VALUE self) {
  rb_need_block();
  return rb_yield(Qnil);
}
static VALUE block_lambda(VALUE self) { return rb_block_lambda(); }
static VALUE this_func(VALUE self) { return ID2SYM(rb_frame_this_func()); }
static VALUE sys_fail(VALUE self, VALUE message) {
  errno = ENOENT;
  rb_sys_fail(NIL_P(message) ? NULL : StringValueCStr(message));
  return Qnil;
}
static VALUE syserr_fail(VALUE self, VALUE number, VALUE message) {
  rb_syserr_fail(NUM2INT(number), NIL_P(message) ? NULL : StringValueCStr(message));
  return Qnil;
}
static VALUE syserr_fail_str(VALUE self, VALUE number, VALUE message) {
  rb_syserr_fail_str(NUM2INT(number), message);
  return Qnil;
}
static VALUE keyword_given(int argc, VALUE *argv, VALUE self) { return rb_keyword_given_p() ? Qtrue : Qfalse; }
static VALUE call_with_keywords(VALUE self, VALUE receiver, VALUE name, VALUE arguments) {
  return rb_funcallv_kw(receiver, SYM2ID(name), RARRAY_LENINT(arguments), RARRAY_PTR(arguments), RB_PASS_KEYWORDS);
}
static VALUE call_without_keywords(VALUE self, VALUE receiver, VALUE name, VALUE arguments) {
  return rb_funcallv_kw(receiver, SYM2ID(name), RARRAY_LENINT(arguments), RARRAY_PTR(arguments), RB_NO_KEYWORDS);
}
static VALUE call_public(VALUE self, VALUE receiver, VALUE name) {
  return rb_funcallv_public(receiver, SYM2ID(name), 0, NULL);
}
static VALUE call_with_block(VALUE self, VALUE receiver, VALUE name, VALUE arguments, VALUE block) {
  return rb_funcall_with_block(receiver, SYM2ID(name), RARRAY_LENINT(arguments), RARRAY_PTR(arguments), block);
}
static VALUE call_with_block_and_keywords(VALUE self, VALUE receiver, VALUE name, VALUE arguments, VALUE block) {
  return rb_funcall_with_block_kw(receiver, SYM2ID(name), RARRAY_LENINT(arguments), RARRAY_PTR(arguments), block,
                                  RB_PASS_KEYWORDS);
}
static VALUE check_call(VALUE self, VALUE receiver, VALUE name) {
  VALUE answered = rb_check_funcall(receiver, SYM2ID(name), 0, NULL);
  return answered == Qundef ? ID2SYM(rb_intern("undefined")) : answered;
}
static VALUE sprintf_values(VALUE self, VALUE values) {
  return rb_f_sprintf(RARRAY_LENINT(values), RARRAY_PTR(values));
}
static VALUE format_values(VALUE self, VALUE format, VALUE values) {
  return rb_str_format(RARRAY_LENINT(values), RARRAY_PTR(values), format);
}
static VALUE backtrace(VALUE self) { return rb_make_backtrace(); }
static VALUE category_warn(VALUE self, VALUE category, VALUE message) {
  rb_category_warn((rb_warning_category_t)NUM2INT(category), "%s", StringValueCStr(message));
  return Qnil;
}
static VALUE map_with_own_block(VALUE self, VALUE array) {
  return rb_block_call(array, rb_intern("map"), 0, NULL, (rb_block_call_func_t)NULL, Qnil);
}
static void write_line(VALUE io) { rb_funcall(io, rb_intern("puts"), 1, rb_str_new_cstr("end proc ran")); }
static VALUE end_proc(VALUE self, VALUE io) {
  rb_set_end_proc(write_line, io);
  return Qnil;
}
static void after_the_vm(ruby_vm_t *vm) {
  puts("vm exit hook ran");
}
static VALUE vm_exit_hook(VALUE self) {
  ruby_vm_at_exit(after_the_vm);
  return Qnil;
}

void Init_c_kernel(void) {
  VALUE cls = rb_define_class("CKernel", rb_cObject);
  rb_define_method(cls, "protect", protect, 1);
  rb_define_method(cls, "protect_then_jump", protect_then_jump, 1);
  rb_define_method(cls, "jump_without_protect", jump_without_protect, 0);
  rb_define_method(cls, "clear_errinfo", clear_errinfo, 0);
  rb_define_method(cls, "rescue", rescue, 2);
  rb_define_method(cls, "rescue2", rescue2, 4);
  rb_define_method(cls, "ensure", ensure, 2);
  rb_define_method(cls, "catch_named", catch_named, 2);
  rb_define_method(cls, "catch_object", catch_object, 2);
  rb_define_method(cls, "throw_named", throw_named, 2);
  rb_define_method(cls, "throw_object", throw_object, 2);
  rb_define_method(cls, "evaluate", evaluate, 1);
  rb_define_method(cls, "evaluate_protected", evaluate_protected, 1);
  rb_define_method(cls, "recurse", recurse, 2);
  rb_define_method(cls, "recurse_raising", recurse_raising, 1);
  rb_define_method(cls, "need_block", need_block, 0);
  rb_define_method(cls, "block_lambda", block_lambda, 0);
  rb_define_method(cls, "this_func", this_func, 0);
  rb_define_method(cls, "sys_fail", sys_fail, 1);
  rb_define_method(cls, "syserr_fail", syserr_fail, 2);
  rb_define_method(cls, "syserr_fail_str", syserr_fail_str, 2);
  rb_define_method(cls, "keyword_given", keyword_given, -1);
  rb_define_method(cls, "call_with_keywords", call_with_keywords, 3);
  rb_define_method(cls, "call_without_keywords", call_without_keywords, 3);
  rb_define_method(cls, "call_public", call_public, 2);
  rb_define_method(cls, "call_with_block", call_with_block, 4);
  rb_define_method(cls, "call_with_block_and_keywords", call_with_block_and_keywords, 4);
  rb_define_method(cls, "check_call", check_call, 2);
  rb_define_method(cls, "sprintf_values", sprintf_values, 1);
  rb_define_method(cls, "format_values", format_values, 2);
  rb_define_method(cls, "backtrace", backtrace, 0);
  rb_define_method(cls, "category_warn", category_warn, 2);
  rb_define_method(cls, "map_with_own_block", map_with_own_block, 1);
  rb_define_method(cls, "end_proc", end_proc, 1);
  rb_define_method(cls, "vm_exit_hook", vm_exit_hook, 0);
  rb_define_const(cls, "FRAME_AT_LOAD", rb_frame_this_func() == 0 ? Qtrue : Qfalse);
}
