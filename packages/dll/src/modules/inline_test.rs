use std::sync::{Arc, Mutex};

use anyhow::Result;

use once_cell::sync::Lazy;
use retour::GenericDetour;

use crate::utils::ipc_println;
use _utils::*;

type HookFnType = extern "C" fn(i32) -> i32;

static hooker: Lazy<Arc<Mutex<Option<GenericDetour<HookFnType>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub fn init_inline_hook() -> Result<()> {
    let mut ipc_server = create_server("rust_winhook_core".to_string())?;

    let msg: Msg = ipc_server.read()?;
    let address = match msg {
        Msg::TransferHookAddress(addr) => addr,
        _ => unreachable!(),
    };

    ipc_println(format!("Hooking function address = {:#X}", address));
    let ori: HookFnType = unsafe { std::mem::transmute(address) };
    hooker.clone().lock().unwrap().replace(unsafe {
        let ret = GenericDetour::new(ori, our_add_42).unwrap();
        ret.enable().unwrap();
        ret
    });

    Ok(())
}

extern "C" fn our_add_42(_input: i32) -> i32 {
    let hooker_inside = hooker.clone();

    unsafe {
        hooker_inside
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .disable()
            .unwrap()
    };

    let ret_val = 233333;
    ipc_println(format!("Inject success"));

    unsafe {
        hooker_inside
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .enable()
            .unwrap()
    };

    ret_val
}
