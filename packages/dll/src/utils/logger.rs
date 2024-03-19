use _utils::Msg;

use super::ipc_client;

pub fn ipc_println(s: impl ToString) {
    ipc_client
        .clone()
        .lock()
        .unwrap()
        .write(&Msg::Log(s.to_string()))
        .unwrap();
}
