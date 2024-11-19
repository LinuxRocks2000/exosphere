/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

pub mod protocol;
pub mod comms;
pub const VERSION : u8 = 1; // bump this up every time a major change is made (overflow at 256; this is not meant to be an authoritative correct version)
pub mod types;
pub mod fab;
pub mod pathfollower;
mod steal_mut;
pub use steal_mut::steal_mut;

use serde_derive::{ Serialize, Deserialize };

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PieceId(u64);
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct PlayerId(pub u64);


impl PlayerId {
    pub const SYSTEM : PlayerId = PlayerId(0);
}


impl std::hash::Hash for PieceId {
    fn hash<H : std::hash::Hasher>(&self, state : &mut H) {
        self.0.hash(state);
    }
}


impl std::hash::Hash for PlayerId {
    fn hash<H : std::hash::Hasher>(&self, state : &mut H) {
        self.0.hash(state);
    }
}


#[cfg(feature="server")]
impl std::convert::From<bevy::prelude::Entity> for PieceId {
    fn from(item : bevy::prelude::Entity) -> Self {
        PieceId(item.to_bits())
    }
}

#[cfg(feature="server")]
impl std::convert::Into<bevy::prelude::Entity> for PieceId {
    fn into(self) -> bevy::prelude::Entity {
        bevy::prelude::Entity::from_bits(self.0)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::{ Serialize, Deserialize };
    use protocol::{ ProtocolSerialize, ProtocolDeserialize };

    #[test]
    fn test_tuple_variant() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        enum Test {
            VariantOne(String, u8),
            VariantTwo(String, u16),
            VariantThree(String, u32)
        }
        let v = Test::VariantTwo ("Hello, World".to_string(), 304);
        let dec = Test::decode(&v.encode().unwrap()).unwrap();
        assert_eq!(dec, v);
    }

    #[test]
    fn test_struct_variant() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        enum Test {
            VariantOne { name : String, age : u8 },
            VariantTwo { name : String, age : u16 },
            VariantThree { name : String, age : u32 }
        }
        let v = Test::VariantTwo { name : "Samuel L. Jackson".to_string(), age : 371 };
        let dec = Test::decode(&v.encode().unwrap()).unwrap();
        assert_eq!(dec, v);
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Test {
            string : String,
            num : u32,
            boolean : bool
        }

        let v = Test {
            string : "Hello, World".to_string(),
            num : 70000,
            boolean : true
        };
        let dec = Test::decode(&v.encode().unwrap()).unwrap();
        assert_eq!(dec, v);
    }

    #[test]
    fn test_some() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Test {
            val1 : Option<String>,
            val2 : Option<u32>
        }
        let v = Test {
            val1 : Some("hello, world".to_string()),
            val2 : None
        };
        let dec = Test::decode(&v.encode().unwrap()).unwrap();
        assert_eq!(v, dec);
    }

    #[test]
    fn test_nested() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        enum TestOuter {
            Value(TestInner)
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestInner {
            val1 : Option<String>,
            val2 : Option<u32>
        }

        let v = TestOuter::Value(TestInner { val1 : Some("hello, world".to_string()), val2 : None });
        let dec = TestOuter::decode(&v.encode().unwrap()).unwrap();
        assert_eq!(v, dec);
    }

    #[test]
    fn test_seq() {
        let v = vec![2, 3, 4, 5, 6, 7];
        let packet = v.encode().unwrap();
        println!("packet: {:?}", packet);
        let dec = Vec::<u8>::decode(&packet).unwrap();
        assert_eq!(v, dec);
    }
}