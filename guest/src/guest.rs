
extern crate wee_alloc;
use std::mem;
use std::slice;
use common::bits_n_pieces::u64_merge_bits;
// use common::bits_n_pieces::u64_split_bits;

extern "C" {
    fn __host_process_string(ptr: u32, cap: u32) -> u64;
    fn __host_copy(host_ptr: u64, guest_ptr: u32, len: u32) -> u64;
}

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub fn host_string(ptr: u32, len: u32) -> String {
    let slice = unsafe { slice::from_raw_parts(ptr as _, len as _) };
    String::from(std::str::from_utf8(slice).unwrap())
}

#[no_mangle]
/// hack to allow the host to allocate a string capacity and receive the pointer to write into it
/// from outside
pub extern "C" fn pre_alloc_string(cap: u32) -> i32 {
    // https://doc.rust-lang.org/std/string/struct.String.html#examples-8
    // Prevent automatically dropping the String's data
    let dummy:Vec<u8> = Vec::with_capacity(cap as _);
    let ptr = dummy.as_slice().as_ptr() as i32;
    mem::ManuallyDrop::new(dummy);
    ptr
}

pub extern "C" fn dealloc_string(ptr: u32, len: u32) {
    // assigning a string straight from ptr and len then dropping it will remind the allocator that
    // this memory exists then immediately drop it
    let _ = host_string(ptr, len);
}

pub fn prepare_return(s: String) -> u64 {
    let s_ptr = s.as_ptr();
    let s_len = s.len();
    mem::ManuallyDrop::new(s);
    u64_merge_bits(s_ptr as _, s_len as _)
}

fn host_process_string(s: String) -> String {
    let host_ptr = unsafe { __host_process_string(s.as_ptr() as _, s.len() as _) };
    let len = 19;
    let guest_ptr = pre_alloc_string(len);
    unsafe { __host_copy(host_ptr, guest_ptr as _, len as _) };
    host_string(guest_ptr as _, len as _)
}

#[no_mangle]
pub extern "C" fn process_string(ptr: u32, cap: u32) -> u64 {
    // get the string the host is trying to pass us out of memory
    // the ptr and cap line up with what was previously allocated with pre_alloc_string
    let s = host_string(ptr, cap);

    // imported host function calls are always unsafe
    let guest_processed = format!("guest: {}", s);
    let host_processed = host_process_string(guest_processed);
    prepare_return(host_processed)
}
