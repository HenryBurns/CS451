use std::slice;
fn main() {
    do_stuff();
}

fn do_stuff() {
    let mut num = 3;

    // two types of raw pointer
    // *const and *mut.
    let r1 : *const i32 = &num;
    let r2 : *mut i32 = &mut num;

    unsafe {
        println!("R1 is {}", *r1);
        println!("R2 is {}", *r2);
    }
}

// We are making a function split_at_mut, which
// splits a splice at a midpoint specified as
// a parameter.
fn split_at_mut(slice: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = slice.len();
    let pointer: *mut i32 = slice.as_mut_ptr();

    assert!(mid <= len);

    // Get slice 0..mid and mid..end
    unsafe {
        (
            slice::from_raw_parts_mut(pointer, mid),
            slice::from_raw_parts_mut(pointer.offset(mid as isize),len - mid),
        )
    }
}
unsafe fn unsafe_function() {

}
