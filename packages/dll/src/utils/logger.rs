use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

use _utils::*;

pub static ipc_client: Lazy<Arc<Mutex<Pipe>>> = Lazy::new(|| {
    let conn = create_client("rust_winhook_demo".to_string()).unwrap();

    Arc::new(Mutex::new(conn))
});

pub fn ipc_println(s: impl ToString) {
    ipc_client
        .clone()
        .lock()
        .unwrap()
        .write(&Msg::Log(s.to_string()))
        .unwrap();
}
