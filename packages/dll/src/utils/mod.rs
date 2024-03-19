mod logger;

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

use _utils::*;

pub use logger::*;

pub static ipc_client: Lazy<Arc<Mutex<Pipe>>> = Lazy::new(|| {
    let conn = create_client("rust_winhook_dll".to_string()).unwrap();

    Arc::new(Mutex::new(conn))
});
