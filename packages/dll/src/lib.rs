#![cfg(windows)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

use _utils::create_client;
use once_cell::sync::Lazy;
use std::{
    ffi::CStr,
    os::raw::c_void,
    sync::{Arc, Mutex},
};

use retour::GenericDetour;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{BOOL, HANDLE, HMODULE},
        System::{
            LibraryLoader::{GetProcAddress, LoadLibraryA},
            SystemServices::{
                DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
            },
        },
    },
};

#[allow(non_snake_case)]
type fn_LoadLibraryA = extern "system" fn(PCSTR) -> HMODULE;

static hook_LoadLibraryA: Lazy<GenericDetour<fn_LoadLibraryA>> = Lazy::new(|| {
    let library_handle = unsafe { LoadLibraryA(PCSTR(b"kernel32.dll\0".as_ptr() as _)) }.unwrap();
    let address = unsafe { GetProcAddress(library_handle, PCSTR(b"LoadLibraryA\0".as_ptr() as _)) };
    let ori: fn_LoadLibraryA = unsafe { std::mem::transmute(address) };
    unsafe { GenericDetour::new(ori, our_LoadLibraryA).unwrap() }
});

use _utils::*;

static ipc_client: Lazy<Arc<Mutex<Pipe>>> = Lazy::new(|| {
    let conn = create_client("rust_winhook_demo".to_string()).unwrap();

    Arc::new(Mutex::new(conn))
});

fn ipc_println(s: impl ToString) {
    ipc_client
        .clone()
        .lock()
        .unwrap()
        .write(&Msg::Log(s.to_string()))
        .unwrap();
}

extern "system" fn our_LoadLibraryA(lpFileName: PCSTR) -> HMODULE {
    let file_name = unsafe { CStr::from_ptr(lpFileName.as_ptr() as _) };
    ipc_println(format!("our_LoadLibraryA lpFileName = {:?}", file_name));
    unsafe { hook_LoadLibraryA.disable().unwrap() };
    let ret_val = hook_LoadLibraryA.call(lpFileName);
    ipc_println(format!(
        "our_LoadLibraryA lpFileName = {:?} ret_val = {:#X}",
        file_name, ret_val.0
    ));
    unsafe { hook_LoadLibraryA.enable().unwrap() };
    ret_val
}

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => unsafe {
            hook_LoadLibraryA.enable().unwrap();
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
