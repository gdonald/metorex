#include "ruby.h"

void Init_c_autoloaded(void) {
  VALUE holder = rb_const_get(rb_cObject, rb_intern("AutoloadHolder"));
  rb_define_module_under(holder, "FromExtension");
}
