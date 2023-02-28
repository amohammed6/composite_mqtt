#include <cos_types.h>

/**
 * These will likely go into their own rust runtime library. rustc creates these shims 
 * at LINK time so that it can optionally hook in functions for externally defined allocators 
 * (like jemalloc) that are defined with #[global_allocator]. We however do not use rustc 
 * for linking so we have to define default ones ourselves... The symbols are alrady defined,
 * we just need to hook them together.
 */

extern u8_t *__rdl_alloc(u64_t, u64_t);
extern void  __rdl_dealloc(u8_t *, u64_t, u64_t);
extern u8_t *__rdl_realloc(u8_t *, u64_t, u64_t, u64_t);
extern u8_t *__rdl_alloc_zeroed(u64_t, u64_t);
extern void  __rdl_oom(u64_t, u64_t);


u8_t *
__rust_alloc(u64_t size, u64_t align) {
    return __rdl_alloc(size, align);
}

void  
__rust_dealloc(u8_t *ptr, u64_t size, u64_t align) {
    __rdl_dealloc(ptr, size, align);
}

u8_t *
__rust_realloc(u8_t *ptr, u64_t old_size, u64_t align, u64_t new_size) {
    return __rdl_realloc(ptr, old_size, align, new_size);
}

u8_t *
__rust_alloc_zeroed(u64_t size, u64_t align) {
    return __rdl_alloc_zeroed(size, align);
}

u8_t __rust_alloc_error_handler_should_panic; 
void 
__rust_alloc_error_handler(u64_t size, u64_t align) {
    __rdl_oom(size, align);
}
