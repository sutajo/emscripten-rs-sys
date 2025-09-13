#![cfg_attr(
    feature = "em_js",
    feature(asm_experimental_arch, macro_metavar_expr_concat)
)]

use emscripten_rs_sys::*;
use std::{ffi::CStr, ptr::null_mut};

#[link(name = "websocket.js")]
unsafe extern "C" {}

const MESSAGE: &CStr = c"Árvíztűrő tükörfúrógép";

unsafe extern "C" fn on_open_callback(
    _event_type: ::std::os::raw::c_int,
    websocket_event: *const EmscriptenWebSocketOpenEvent,
    _user_data: *mut ::std::os::raw::c_void,
) -> bool {
    println!("Websocket connection is open");

    unsafe {
        emscripten_websocket_send_utf8_text((*websocket_event).socket, MESSAGE.as_ptr());
    }

    true
}

unsafe extern "C" fn on_message_callback(
    _event_type: ::std::os::raw::c_int,
    websocket_event: *const EmscriptenWebSocketMessageEvent,
    _user_data: *mut ::std::os::raw::c_void,
) -> bool {
    let msg = unsafe { *websocket_event };

    if msg.isText {
        let text = unsafe { CStr::from_ptr(msg.data as _) };
        println!("Message received: {}", text.to_str().unwrap());

        if text == MESSAGE {
            // Got back the echo, shut it down
            unsafe {
                emscripten_websocket_close(msg.socket, 0, c"".as_ptr());
            }
        }
    }

    true
}

fn main() {
    unsafe {
        if !emscripten_websocket_is_supported() {
            eprintln!("Websockets are not supported.");
            return;
        }

        let mut attributes = EmscriptenWebSocketCreateAttributes {
            url: c"wss://echo.websocket.org".as_ptr(),
            ..Default::default()
        };

        let ws = emscripten_websocket_new(&mut attributes);
        emscripten_websocket_set_onopen_callback_on_thread(
            ws,
            null_mut(),
            Some(on_open_callback),
            1,
        );
        emscripten_websocket_set_onmessage_callback_on_thread(
            ws,
            null_mut(),
            Some(on_message_callback),
            1,
        );

        emscripten_exit_with_live_runtime()
    }
}
