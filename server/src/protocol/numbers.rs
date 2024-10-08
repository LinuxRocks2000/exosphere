/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

use crate::protocol::Protocol;
use crate::protocol::DecodeError;


impl Protocol for u8 {
    fn size(&self) -> usize {
        1
    }

    fn encode_into(&self, data : &mut [u8]) {
        data[0] = *self;
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        data.get(0).ok_or(DecodeError{}).copied()
    }
}

impl Protocol for u16 {
    fn size(&self) -> usize {
        2
    }

    fn encode_into(&self, data : &mut [u8]) {
        let b = self.to_le_bytes();
        data[0] = b[0];
        data[1] = b[1];
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_le_bytes(match data.get(0..2).ok_or(DecodeError{})?.try_into() {
            Ok(data) => data,
            Err(_) => {return Err(DecodeError{});}
        }))
    }
}

impl Protocol for u32 {
    fn size(&self) -> usize {
        4
    }

    fn encode_into(&self, data : &mut [u8]) {
        let b = self.to_le_bytes();
        for x in 0..4 {
            data[x] = b[x];
        }
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_le_bytes(match data.get(0..4).ok_or(DecodeError{})?.try_into() {
            Ok(data) => data,
            Err(_) => {return Err(DecodeError{});}
        }))
    }
}

impl Protocol for u64 {
    fn size(&self) -> usize {
        8
    }

    fn encode_into(&self, data : &mut [u8]) {
        let b = self.to_le_bytes();
        for x in 0..8 {
            data[x] = b[x];
        }
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_le_bytes(match data.get(0..8).ok_or(DecodeError{})?.try_into() {
            Ok(data) => data,
            Err(_) => {return Err(DecodeError{});}
        }))
    }
}


impl Protocol for i8 {
    fn size(&self) -> usize {
        1
    }

    fn encode_into(&self, data : &mut [u8]) {
        data[0] = self.to_le_bytes()[0];
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_le_bytes([*data.get(0).ok_or(DecodeError{})?]))
    }
}

impl Protocol for i16 {
    fn size(&self) -> usize {
        2
    }

    fn encode_into(&self, data : &mut [u8]) {
        let b = self.to_le_bytes();
        data[0] = b[0];
        data[1] = b[1];
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_le_bytes(match data.get(0..2).ok_or(DecodeError{})?.try_into() {
            Ok(data) => data,
            Err(_) => {return Err(DecodeError{});}
        }))
    }
}

impl Protocol for i32 {
    fn size(&self) -> usize {
        4
    }

    fn encode_into(&self, data : &mut [u8]) {
        let b = self.to_le_bytes();
        for x in 0..4 {
            data[x] = b[x];
        }
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_le_bytes(match data.get(0..4).ok_or(DecodeError{})?.try_into() {
            Ok(data) => data,
            Err(_) => {return Err(DecodeError{});}
        }))
    }
}

impl Protocol for i64 {
    fn size(&self) -> usize {
        8
    }

    fn encode_into(&self, data : &mut [u8]) {
        let b = self.to_le_bytes();
        for x in 0..8 {
            data[x] = b[x];
        }
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_le_bytes(match data.get(0..8).ok_or(DecodeError{})?.try_into() {
            Ok(data) => data,
            Err(_) => {return Err(DecodeError{});}
        }))
    }
}

impl Protocol for f32 {
    fn size(&self) -> usize {
        4
    }

    fn encode_into(&self, data : &mut [u8]) {
        let b = self.to_le_bytes();
        for x in 0..4 {
            data[x] = b[x];
        }
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_le_bytes(match data.get(0..4).ok_or(DecodeError{})?.try_into() {
            Ok(data) => data,
            Err(_) => {return Err(DecodeError{});}
        }))
    }
}

impl Protocol for f64 {
    fn size(&self) -> usize {
        8
    }

    fn encode_into(&self, data : &mut [u8]) {
        let b = self.to_le_bytes();
        for x in 0..8 {
            data[x] = b[x];
        }
    }

    fn decode_from(data : &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_le_bytes(match data.get(0..8).ok_or(DecodeError{})?.try_into() {
            Ok(data) => data,
            Err(_) => {return Err(DecodeError{});}
        }))
    }
}