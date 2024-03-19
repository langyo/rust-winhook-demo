use anyhow::Result;

use crate::utils::ipc_println;
use _utils::*;

pub fn init_inline_hook() -> Result<()> {
    let mut ipc_server = create_server("rust_winhook_core".to_string())?;

    let msg: Msg = ipc_server.read()?;
    let address = match msg {
        Msg::TransferHookAddress(addr) => addr,
        _ => unreachable!(),
    };

    ipc_println(format!("Hooked function address = {:#X}", address));

    Ok(())
}
