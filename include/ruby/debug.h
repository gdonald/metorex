#ifndef RUBY_DEBUG_H
#define RUBY_DEBUG_H 1

#include "ruby/ruby.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t rb_event_flag_t;

#define RUBY_EVENT_NONE 0x0000
#define RUBY_EVENT_LINE 0x0001
#define RUBY_EVENT_CLASS 0x0002
#define RUBY_EVENT_END 0x0004
#define RUBY_EVENT_CALL 0x0008
#define RUBY_EVENT_RETURN 0x0010
#define RUBY_EVENT_C_CALL 0x0020
#define RUBY_EVENT_C_RETURN 0x0040
#define RUBY_EVENT_RAISE 0x0080
#define RUBY_EVENT_ALL 0x00ff
#define RUBY_EVENT_B_CALL 0x0100
#define RUBY_EVENT_B_RETURN 0x0200
#define RUBY_EVENT_THREAD_BEGIN 0x0400
#define RUBY_EVENT_THREAD_END 0x0800
#define RUBY_EVENT_FIBER_SWITCH 0x1000
#define RUBY_EVENT_SCRIPT_COMPILED 0x2000
#define RUBY_EVENT_RESCUE 0x4000
#define RUBY_EVENT_TRACEPOINT_ALL 0xffff

VALUE rb_tracepoint_new(VALUE target_thread, rb_event_flag_t events, void (*func)(VALUE, void *),
                        void *data);
VALUE rb_tracepoint_enable(VALUE trace);
VALUE rb_tracepoint_disable(VALUE trace);
VALUE rb_tracepoint_enabled_p(VALUE trace);

#ifdef __cplusplus
}
#endif

#endif
