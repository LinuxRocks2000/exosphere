/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

use crate::Protocol;
use crate::protocol::DecodeError;

impl Protocol for String {
    fn size(&self) -> usize {
        self.len() + 2 // 16-bit string length identifiers: I doubt we'll go above 65kb
        // richard, if you read this, please know that there _are_ frame validity checks and a size cap, you won't crash the server by spamming >65 kilobytes of bad strings
        // instead of wasting your time and my bandwidth doing _that_, how about you try forcing a buffer overflow or something
    }

    fn encode_into(&self, buffer : &mut [u8]) {
        // encode the string length in LITTLE ENDIAN
        // everything is LE for ezpzlmnskweezy
        let bytes = (self.len() as u16).to_le_bytes();
        buffer[0] = bytes[0];
        buffer[1] = bytes[1];
        for (i, x) in self.as_bytes().iter().enumerate() {
            buffer[i + 2] = *x;
        }
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        let len = u16::from_le_bytes(match data.get(0..2).ok_or(DecodeError{})?.try_into() {
            Ok(data) => data,
            Err(_) => {return Err(DecodeError{});}
        }) as usize;
        match std::str::from_utf8(&data.get(2..(len + 2)).ok_or(DecodeError{})?) {
            Ok(data) => Ok(data.to_string()),
            Err(_) => Err(DecodeError{})
        }
    }
}