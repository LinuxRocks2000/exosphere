/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// serde-based protocol serialization and deserialization

pub mod serializer;
pub mod deserializer;
pub use serializer::ProtocolSerialize;
pub use deserializer::ProtocolDeserialize;

use std::fmt;
use std::fmt::Display;



#[derive(Debug)]
pub enum Error {
    BadInteger,
    BadFloat,
    BadString,
    UnsizedSequence,
    UnsizedMap,
    InvalidType,
    Custom(String)
}


impl serde::ser::Error for Error {
    fn custom<T : Display>(msg : T) -> Self {
        Self::Custom(msg.to_string())
    }
}


impl serde::de::Error for Error {
    fn custom<T : Display>(msg : T) -> Self {
        Self::Custom(msg.to_string())
    }
}


impl Display for Error {
    fn fmt(&self, formatter : &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&format!("protocol encode/decode error {:?}", self))
    }
}


impl std::error::Error for Error {}


pub type Result = std::result::Result<(), Error>;
pub type Result2<T> = std::result::Result<T, Error>;
