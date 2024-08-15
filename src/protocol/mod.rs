pub mod traits;
pub mod numbers;
pub mod misc;

pub use traits::Protocol;
pub use traits::ProtocolRoot;
pub use derive_macro::*;


// shameless copy from the obsolete protocol_v3
#[derive(Debug)]
pub struct DecodeError {}


impl std::error::Error for DecodeError {
    fn description(&self) -> &str {
        "Bad protocol frame received from a client!"
    }
}


impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Protocol Decode Error")
    }
}

