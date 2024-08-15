/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

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

