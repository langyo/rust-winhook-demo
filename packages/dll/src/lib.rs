#![cfg(windows)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

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

use interprocess::local_socket::LocalSocketStream;
use std::io::{prelude::*, BufReader};

#[allow(non_snake_case)]
type fn_LoadLibraryA = extern "system" fn(PCSTR) -> HMODULE;

static hook_LoadLibraryA: Lazy<GenericDetour<fn_LoadLibraryA>> = Lazy::new(|| {
    let library_handle = unsafe { LoadLibraryA(PCSTR(b"kernel32.dll\0".as_ptr() as _)) }.unwrap();
    let address = unsafe { GetProcAddress(library_handle, PCSTR(b"LoadLibraryA\0".as_ptr() as _)) };
    let ori: fn_LoadLibraryA = unsafe { std::mem::transmute(address) };
    unsafe { GenericDetour::new(ori, our_LoadLibraryA).unwrap() }
});

static ipc_client: Lazy<Arc<Mutex<BufReader<LocalSocketStream>>>> = Lazy::new(|| {
    let name = "/tmp/rust_winhook_demo.sock";

    let conn = LocalSocketStream::connect(name).unwrap();
    let conn = BufReader::new(conn);

    Arc::new(Mutex::new(conn))
});

fn ipc_println(s: impl ToString) {
    ipc_client
        .clone()
        .lock()
        .unwrap()
        .get_mut()
        .write_all(s.to_string().as_bytes())
        .unwrap();

    let mut buffer = String::with_capacity(128);
    ipc_client
        .clone()
        .lock()
        .unwrap()
        .read_line(&mut buffer)
        .unwrap();
    println!("IPC Server answered: {}", buffer);
}

extern "system" fn our_LoadLibraryA(lpFileName: PCSTR) -> HMODULE {
    let file_name = unsafe { CStr::from_ptr(lpFileName.as_ptr() as _) };
    println!("our_LoadLibraryA lpFileName = {:?}", file_name);
    ipc_println(format!(
        "IPC our_LoadLibraryA lpFileName = {:?}\n",
        file_name
    ));
    unsafe { hook_LoadLibraryA.disable().unwrap() };
    let ret_val = hook_LoadLibraryA.call(lpFileName);
    println!(
        "our_LoadLibraryA lpFileName = {:?} ret_val = {:#X}",
        file_name, ret_val.0
    );
    ipc_println(format!(
        "IPC our_LoadLibraryA lpFileName = {:?} ret_val = {:#X}\n",
        file_name, ret_val.0
    ));
    unsafe { hook_LoadLibraryA.enable().unwrap() };
    ret_val
}

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            // TODO - Use `interprocess` to communicate with the injector by named pipe
            println!("DLL attached");
            unsafe {
                hook_LoadLibraryA.enable().unwrap();
            }
        }
        DLL_PROCESS_DETACH => {
            println!("DLL detached");
            ipc_println("stop\n");
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        _ => {}
    };
    BOOL::from(true)
}
