use std::io::Read as _;
use windows::{core::*, Win32::System::LibraryLoader::LoadLibraryA};

use _utils::*;

#[no_mangle]
#[inline(never)]
pub extern "C" fn add_42(x: i32) -> i32 {
    x + 42
}

fn main() {
    println!("[core] Waiting 1s for hook");
    std::thread::sleep(std::time::Duration::from_millis(1000));

    println!("[core] Loading kernel32.dll");
    unsafe {
        let _ = LoadLibraryA(PCSTR(b"kernel32.dll\0".as_ptr() as _));
    }
    println!("[core] Loaded kernel32.dll");

    println!("[core] Inline hooking add_42 test begin");
    println!("[core] add_42's address is 0x{:x}", add_42 as usize);

    let mut ipc_service = create_client("rust_winhook_core".to_string()).unwrap();
    println!("[core] Connected to dll IPC server");

    ipc_service
        .write(&Msg::TransferHookAddress(add_42 as usize))
        .unwrap();

    println!("[core] Waiting 1s for hook");
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let result = add_42(100);
    println!("[core] add_42(100) = {}", result);

    println!("[core] Inline hooking add_42 test end");

    println!("[core] Press any key to exit...");
    let _ = std::io::stdin().read(&mut [0u8]).unwrap();
}
