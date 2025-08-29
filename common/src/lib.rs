/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

pub mod comms;
pub const VERSION: u8 = 2; // bump this up every time a major change is made (overflow at 256; this is not meant to be an authoritative correct version)
pub mod fab;
pub mod pathfollower;
mod steal_mut;
pub mod types;
pub use steal_mut::steal_mut;

use bitcode::{Decode, Encode};

#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, Eq)]
pub struct PieceId(u64);
#[derive(Copy, Clone, Encode, Decode, Debug, PartialEq, Eq)]
pub struct PlayerId(pub u64);

impl PlayerId {
    pub const SYSTEM: PlayerId = PlayerId(0);
}

impl PieceId {
    pub const ZERO: PieceId = PieceId(0);
}

impl std::hash::Hash for PieceId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl std::hash::Hash for PlayerId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[cfg(feature = "server")]
impl std::convert::From<bevy::prelude::Entity> for PieceId {
    fn from(item: bevy::prelude::Entity) -> Self {
        PieceId(item.to_bits())
    }
}

#[cfg(feature = "server")]
impl std::convert::Into<bevy::prelude::Entity> for PieceId {
    fn into(self) -> bevy::prelude::Entity {
        bevy::prelude::Entity::from_bits(self.0)
    }
}
