/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// ALL of the component declarations and their respective impls

use bevy::prelude::Component;
use crate::Bullets;
use crate::ExplosionProperties;
use bevy::prelude::Vec2;
use crate::PlaceType;


#[derive(Component)]
pub struct GamePiece {
    pub type_indicator : u16, // the type indicator sent to the client
    // assigned by the gamepiece builder functions
    // todo: do this a better way
    pub owner : u64, // entry in the Clients hashmap
    pub slot : u8, // identity slot of the owner
    // in the future, we may want to eliminate this and instead do lookups in the HashMap (which is swisstable, so it's pretty fast)
    // but for now it's convenient
    pub last_update_pos : Vec2,
    pub last_update_ang : f32,
    pub health : f32
}


impl GamePiece {
    pub fn new(type_indicator : u16, owner : u64, slot : u8, health : f32) -> Self {
        Self {
            type_indicator,
            owner,
            slot,
            last_update_pos : Vec2 {
                x : 0.0,
                y : 0.0
            },
            last_update_ang : 0.0,
            health : health
        }
    }
}


#[derive(Component)]
pub struct Territory { // a territory control radius produced by a castle or fort.
    pub radius : f32
}


impl Territory {
    pub fn castle() -> Self { // TODO: make this meaningful
        Self {
            radius : 600.0
        }
    }
}


#[derive(Component)]
pub struct Fabber { // a fabber bay with a radius
    pub radius : f32,
    pub l_missiles : u8,
    pub l_ships : u8,
    pub l_econ : u8,
    pub l_defense : u8,
    pub l_buildings : u8
}



impl Fabber {
    pub fn castle() -> Self {
        Self { // Large-M4S2E2D3B2
            radius : 500.0,
            l_missiles : 4,
            l_ships : 3,
            l_econ : 2,
            l_defense : 3,
            l_buildings : 2
        }
    }

    pub fn is_available(&self, tp : PlaceType) -> bool { // determine if this fabber can produce an object given its numerical identifier
        match tp {
            PlaceType::BasicFighter => self.l_ships >= 1,
            PlaceType::TieFighter => self.l_ships >= 1,
            PlaceType::Castle => false, // fabbers can never place castles
            PlaceType::Sniper => self.l_ships >= 1,
            PlaceType::DemolitionCruiser => self.l_ships >= 2,
            PlaceType::Battleship => self.l_ships >= 3,
            PlaceType::Seed => self.l_econ >= 1,
            PlaceType::Chest => false // fabbers can never place chests
        }
    }
}


#[derive(Component)]
pub struct Ship {
    pub speed : f32,
    pub acc_profile : f32 // in percentage of speed
}


impl Ship {
    pub fn normal() -> Self {
        return Self {
            speed : 16.0,
            acc_profile : 0.33
        }
    }

    pub fn fast() -> Self {
        return Self {
            speed : 32.0,
            acc_profile : 0.5
        }
    }

    pub fn slow() -> Self {
        return Self {
            speed : 12.0,
            acc_profile : 0.33
        }
    }
}


#[derive(Component)]
pub struct TimeToLive {
    pub lifetime : u16
}


#[derive(Component)]
pub struct Bullet {
    pub tp : Bullets
} // bullet collision semantics
// normal collisions between entities are only destructive if greater than a threshold
// bullet collisions are always destructive


#[derive(Component)]
pub struct Seed {
    pub time_to_grow : u16 // remaining time before this seed sprouts
}

impl Seed {
    pub fn new() -> Self {
        Self {
            time_to_grow : 600
        }
    }
}


#[derive(Component)]
pub struct Chest {}


#[derive(Component)]
pub struct Gun {
    pub enabled : bool,
    pub cd : u16, // cooldown ticks between shots
    pub bullets : Bullets,
    pub repeats : u16, // number of repeater shots
    pub repeat_cd : u16, // time between repeater shots
    // state fields (don't touch):
    pub r_point : u16, // current repeater position
    // the repeat pattern is pretty simple. when a bullet is fired, r_point is incremented by one, and if it's less than the number of repeats, `tick` is set to
    // repeat_cd instead of cd. when r_point >= repeats, r_point = 0 and tick = cd.
    pub tick : u16, // current tick
    pub barrels : u16,
    pub barrel_spacing : f32,
    pub center_offset : f32
}


impl Gun {
    pub fn mediocre() -> Self {
        Self {
            enabled : true,
            cd : 20,
            bullets : Bullets::MinorBullet(40),
            repeats : 0,
            repeat_cd : 0,
            r_point : 0,
            tick : 1,
            barrels : 1,
            barrel_spacing : 0.0,
            center_offset : 40.0
        }
    }

    pub fn basic_repeater(repeats : u16) -> Self {
        Self {
            enabled : true,
            cd : 25,
            bullets : Bullets::MinorBullet(50),
            repeats,
            repeat_cd : 5,
            r_point : 0,
            tick : 1,
            barrels : 1,
            barrel_spacing : 0.0,
            center_offset : 40.0
        }
    }

    pub fn sniper() -> Self {
        Self {
            enabled : true,
            cd : 150,
            bullets : Bullets::MinorBullet(200),
            repeats : 0,
            repeat_cd : 0,
            r_point : 0,
            tick : 1,
            barrels : 1,
            barrel_spacing : 0.0,
            center_offset : 40.0
        }
    }

    pub fn bomber() -> Self {
        Self {
            enabled : true,
            cd : 120,
            bullets : Bullets::Bomb(ExplosionProperties::small(), 75),
            repeats : 0,
            repeat_cd : 0,
            r_point : 0,
            tick : 1,
            barrels : 1,
            barrel_spacing : 0.0,
            center_offset : 40.0
        }
    }

    pub fn extended_barrels(mut self, num : u16, spacing : f32) -> Self {
        self.barrels += num;
        self.barrel_spacing = spacing;
        self
    }

    pub fn offset(mut self, off : f32) -> Self {
        self.center_offset = off;
        self
    }
}