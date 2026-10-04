#ifndef RUBY_VM_H
#define RUBY_VM_H 1

#include "ruby/ruby.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ruby_vm_struct ruby_vm_t;
void ruby_vm_at_exit(void (*function)(ruby_vm_t *vm));

#ifdef __cplusplus
}
#endif

#endif
