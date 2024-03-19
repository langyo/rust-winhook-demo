#![cfg(windows)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

mod modules;
mod utils;

use std::os::raw::c_void;

use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::SystemServices::{
        DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
    },
};

use _utils::*;
use utils::*;

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => unsafe {
            modules::eat_test::hooker.enable().unwrap();

            std::thread::spawn(|| {
                modules::inline_test::init_inline_hook().unwrap();
            });
        },
        DLL_PROCESS_DETACH => {
            ipc_client
                .clone()
                .lock()
                .unwrap()
                .write(&Msg::Terminated)
                .unwrap();
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        _ => {}
    };
    BOOL::from(true)
}
