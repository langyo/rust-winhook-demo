mod pipe;

use serde::{Deserialize, Serialize};

pub use pipe::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Msg {
    Log(String),
    Terminated,
}
