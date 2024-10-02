/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

use crate::protocol::DecodeError;

// root of a protocol (this is what you derive)
pub trait ProtocolRoot : Protocol {
    fn report_tag(&self) -> u16; // get the outer tag for this frame

    fn encode(&self) -> Vec<u8> { // convenient wrapper for encode_into
        let mut result = vec![0u8; self.size()];
        self.encode_into(&mut result);
        result
    }
}

// protocol serializable thing
pub trait Protocol {
    fn size(&self) -> usize; // get the size of this protocol entry

    fn encode_into(&self, data : &mut [u8]); // write this protocol entry to an &mut [u8] buffer (guaranteed to be at least as large as size() reports)
    // encoding can't fail; if data isn't encodable, it'll be caught by the compiler
    // the only imaginable error is an allocation failure, which will just explode nonetheless

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> where Self : Sized; // data may be any size; it's up to the decoder to verify that the data it contains is sufficiently long and is valid
}
