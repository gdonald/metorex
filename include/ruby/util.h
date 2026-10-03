#ifndef RUBY_UTIL_H
#define RUBY_UTIL_H 1

#include "ruby/ruby.h"

#ifdef __cplusplus
extern "C" {
#endif

double ruby_strtod(const char *text, char **end);
#undef strtod
#define strtod(text, end) ruby_strtod((text), (end))

#ifdef __cplusplus
}
#endif

#endif
