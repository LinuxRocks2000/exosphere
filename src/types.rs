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
use num_traits::FromPrimitive;
use crate::components::FabLevels;


#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive)]
pub(crate) enum PieceType {
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
    TrackingMissile = 15
}


impl PieceType {
    pub(crate) fn price(&self) -> u32 {
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
            _ => 0
        }
    }

    pub(crate) fn user_placeable(&self) -> bool {
        match self {
            Self::BasicFighter | Self::TieFighter | Self::Sniper |
            Self::DemolitionCruiser | Self::Battleship | Self::Seed | Self::TrackingMissile |
            Self::Farmhouse | Self::BallisticMissile | Self::SeekingMissile | Self::HypersonicMissile |
            Self::FleetDefenseShip => true, // if you want a type to be user placeable, just add it to this lil' blob.
            _ => false
        }
    }

    pub(crate) fn fabber(&self) -> FabLevels {
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
            _ => FabLevels::default()
        }
    }
}