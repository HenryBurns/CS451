// #![no_std]

use core::panic::PanicInfo;
use core::slice::from_raw_parts_mut;

// Set some WebAssembly  memory to a value
// that JS can read from
#[no_mangle]
pub fn memory_from_wasm_to_js() {
    let wasm_memory: &mut [u8];

    unsafe {
        wasm_memory = from_raw_parts_mut::<u8>(0 as *mut u8, 1);
    }

    wasm_memory[0] =255;
}

/*
#[panic_handler]
fn panic(_info: &PanicInfo) -> !{
    loop{}
}
*/

// bring in the allocater api functions
use std::alloc::{alloc, dealloc, Layout};
use std::slice;
use std::mem;

// Write our own malloc for the JS side of things, to allocate wasm memory
#[no_mangle]
pub fn malloc(size: usize) -> *mut u8 {
    let alignment = std::mem::align_of::<usize>();

    // ::from_size_align returns an option
    if let Ok(layout) = Layout::from_size_align(size, alignment) {
        // we need to allocate our memory here (unsafe)
        unsafe  {
            if layout.size() > 0 {
                let pointer: *mut u8 = alloc(layout);

                // Check for null pointer bc this is unsafe code
                if !pointer.is_null() {
                    return pointer;
                }
            }// else {
             //   return alignment as *mut u8
             //}
        }
    }

    // We get here if the above fails
    std::process::abort();
}

#[no_mangle]
pub fn sum(data: *mut u8, len: usize) -> i32 {
    let array_set_by_js = unsafe {
        std::slice::from_raw_parts(data as *const u8, len)
    };

    let mut total = 0;

    for i in array_set_by_js {
        total += *i;
    }

    return total as i32;

}
