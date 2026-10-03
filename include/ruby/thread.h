#ifndef METOREX_RUBY_THREAD_H
#define METOREX_RUBY_THREAD_H 1

#include "ruby/ruby.h"

typedef void rb_unblock_function_t(void *);
typedef VALUE rb_blocking_function_t(void *);

#define RUBY_UBF_IO ((rb_unblock_function_t *)-1)
#define RUBY_UBF_PROCESS ((rb_unblock_function_t *)-1)

void *rb_thread_call_without_gvl(void *(*function)(void *), void *data,
                                 rb_unblock_function_t *unblock, void *unblock_data);
int ruby_thread_has_gvl_p(void);

#endif
