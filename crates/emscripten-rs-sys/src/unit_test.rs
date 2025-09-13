use crate::*;
use std::ffi::CStr;

#[test]
fn exec_script_string() {
    let result = unsafe {
        CStr::from_ptr(emscripten_run_script_string(
            c"'abc'.toUpperCase()".as_ptr(),
        ))
    };
    assert_eq!(result, c"ABC")
}

#[test]
fn exec_script_int() {
    let result = unsafe { emscripten_run_script_int(c"6*5".as_ptr()) };
    assert_eq!(result, 30)
}

static mut COUNTER: i32 = 0;

unsafe extern "C" fn update() {
    unsafe {
        COUNTER += 1;
        if COUNTER > 10 {
            emscripten_cancel_main_loop();
        }
    }
}

extern "C" fn onexit() {
    assert_eq!(unsafe { COUNTER }, 11)
}

#[test]
fn main_loop() {
    unsafe {
        libc::atexit(onexit);
        emscripten_set_main_loop(Some(update), 30, false);
    }
}
