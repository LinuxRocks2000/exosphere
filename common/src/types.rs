/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// Piece type definitions and helper functions.

/* HOW TO ADD NEW TYPES
  In the client
    1. Draw the sprites for your type. You have quite a bit of freedom here; the typical pattern is a type_name_friendly and a type_name_enemy sprite with appropriate colors
       (see the top of main.js in the client).
    2. Import assets. Inside index.html there is a hidden div#res. Put <img>s for all of your sprites in there, with IDs corresponding to the filename (for instance,
       res/cruise_missile_friendly.svg would have the id cruise_missile_friendly). 
    3. Add a drawing overload. There's a large block of if statements in main.js::mainloop, inside a forEach over every piece on the board. Each one of these corresponds
       to a type identifier (TODO: clean up main.js). The getRes function grabs an image by id from the div#res, and caches them in a JavaScript object for very fast future
       accesses. While not particularly more useful than document.getElementById, reliable use of getRes allows for a lazy-loading paradigm to be implemented eventually with
       minimal pain, so be sure to use it. the fString variable contains either "enemy" or "friendly" and should be directly appended to your type id, like
       `"cruise_missile_" + fString`.
    4. IF you want it to be user placeable, add it to the piece selector block. There's a template in the html.
    5. IF you want it to have standard control semantics (control nodes), add its type id to function canUpdateStrategy in main.js.
  In the backend
    1. Add the variant for your type to the PieceType enum. Make sure it's directly tagged with a number; put it at the bottom of the list regardless of logical positioning
       so it stays clear which numbers correspond to which types for other contributors.
    2. IF you want it to be user placeable, add it to the match statements in PieceType::price, PieceType::user_placeable, and PlaceType::fabber.
       It's important that you don't miss any of these; they default to not user placeable with a price of 0, which can cause bugs if neglected.
    3. Add it to the match statement in main.rs::make_thing. This should at the MINIMUM create a collider with the size of the units (Collider::cuboid uses half-lengths, so
       make sure to divide by 2 on both dimensions lest your pieces be four times as large as expected). For it to be useful, you'll usually want some other components - 
       see components.rs for what's already available, and add your own there if you see fit. If you want very custom semantics, you'll need to either create some new
       systems or modify already-extant systems; get familiar with Bevy ECS and the Exosphere source before doing so.

  Make sure to document your new type in the `techtree` file in the project root. It should always be the authoritative source on game mechanics.
*/

use num_derive::FromPrimitive;
use crate::fab::FabLevels;
#[cfg(feature = "server")]
use bevy_rapier2d::prelude::Collider;
use serde_derive::{ Serialize, Deserialize };


#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive, Serialize, Deserialize)]
pub enum PieceType {
    BasicFighter = 0,
    Castle = 1,
    Bullet = 2,
    TieFighter = 3,
    Sniper = 4,
    DemolitionCruiser = 5,
    Battleship = 6,
    SmallBomb = 7,
    Seed = 8,
    Chest = 9,
    Farmhouse = 10,
    BallisticMissile = 11,
    FleetDefenseShip = 12,
    SeekingMissile = 13,
    HypersonicMissile = 14,
    TrackingMissile = 15,
    CruiseMissile = 16
}


pub enum Asset { // drawing assets, specifically for client side
    Simple(&'static str), // one image
    Partisan(&'static str, &'static str), // (friendly, enemy) for types that have different assets depending on their friendliness
    Unimpl // we don't have any resources for this asset
}


pub enum Shape {
    Box(f32, f32), // width, height
    Unimpl // no shape data for this asset
}


impl Shape {
    pub fn to_bbox(&self) -> (f32, f32) {
        match self {
            Self::Box(w, h) => (*w, *h),
            Self::Unimpl => (50.0, 50.0)
        }
    }

    #[cfg(feature = "server")]
    pub fn to_collider(&self) -> Collider {
        match self {
            Self::Box(w, h) => Collider::cuboid(*w / 2.0, *h / 2.0),
            Self::Unimpl => Collider::cuboid(25.0, 25.0) // a bigass "loading failed" that kills things
        }
    }
}


impl PieceType {
    pub fn price(&self) -> u32 {
        match self {
            Self::BasicFighter => 10,
            Self::TieFighter => 20,
            Self::Sniper => 30,
            Self::FleetDefenseShip => 50,
            Self::DemolitionCruiser => 60,
            Self::Battleship => 200,
            Self::Seed => 5,
            Self::BallisticMissile => 5,
            Self::Farmhouse => 70,
            Self::SeekingMissile => 10,
            Self::HypersonicMissile => 20,
            Self::TrackingMissile => 30,
            Self::CruiseMissile => 50,
            _ => 0
        }
    }

    pub fn user_placeable(&self) -> bool {
        match self {
            Self::BasicFighter | Self::TieFighter | Self::Sniper | Self::CruiseMissile |
            Self::DemolitionCruiser | Self::Battleship | Self::Seed | Self::TrackingMissile |
            Self::Farmhouse | Self::BallisticMissile | Self::SeekingMissile | Self::HypersonicMissile |
            Self::FleetDefenseShip => true, // if you want a type to be user placeable, just add it to this lil' blob.
            _ => false
        }
    }

    pub fn user_movable(&self) -> bool {
        match self {
            Self::BasicFighter | Self::TieFighter | Self::Sniper | Self::CruiseMissile |
            Self::DemolitionCruiser | Self::Battleship | Self::Seed | Self::TrackingMissile |
            Self::Farmhouse | Self::BallisticMissile | Self::SeekingMissile | Self::HypersonicMissile |
            Self::FleetDefenseShip => true, // if you want a type to be movable, just add it to this lil' blob.
            _ => false
        }
    }

    pub fn fabber(&self) -> FabLevels {
        match self {
            Self::BasicFighter => FabLevels::ships(1),
            Self::TieFighter => FabLevels::ships(1),
            Self::Sniper => FabLevels::ships(1),
            Self::DemolitionCruiser => FabLevels::ships(2),
            Self::Battleship => FabLevels::ships(3),
            Self::Seed => FabLevels::econ(1),
            Self::Farmhouse => FabLevels::econ(2),
            Self::BallisticMissile => FabLevels::missiles(1),
            Self::SeekingMissile => FabLevels::missiles(1),
            Self::HypersonicMissile => FabLevels::missiles(2),
            Self::TrackingMissile => FabLevels::missiles(3),
            Self::CruiseMissile => FabLevels::missiles(4),
            _ => FabLevels::default()
        }
    }

    pub fn asset(&self) -> Asset { // specifically for the client side: get the image(s?) for the thing we're drawing
        match self {
            Self::BasicFighter => Asset::Partisan("basic_fighter_friendly.svg", "basic_fighter_enemy.svg"),
            Self::Castle => Asset::Partisan("castle_friendly.svg", "castle_enemy.svg"),
            Self::Bullet => Asset::Simple("bullet.svg"),
            Self::TieFighter => Asset::Partisan("tie_fighter_friendly.svg", "tie_fighter_enemy.svg"),
            Self::Sniper => Asset::Partisan("sniper_friendly.svg", "sniper_enemy.svg"),
            _ => Asset::Unimpl
        }
    }

    pub fn shape(&self) -> Shape {
        match self {
            Self::BasicFighter => Shape::Box(41.0, 41.0),
            Self::Bullet => Shape::Box(5.0, 5.0),
            Self::Castle => Shape::Box(60.0, 60.0),
            Self::TieFighter => Shape::Box(40.0, 50.0),
            Self::Sniper => Shape::Box(60.0, 30.0),
            _ => Shape::Unimpl
        }
    }
}