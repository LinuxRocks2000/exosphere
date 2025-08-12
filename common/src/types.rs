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

use crate::fab::FabLevels;
#[cfg(feature = "server")]
use avian2d::prelude::*;
use bitcode::{Decode, Encode};
use num_derive::FromPrimitive;

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive, Encode, Decode)]
pub enum PieceType {
    BasicFighter,      // impl
    Castle,            // impl
    Bullet,            // impl
    TieFighter,        // impl
    Sniper,            // impl
    DemolitionCruiser, // impl
    Battleship,        // impl
    SmallBomb,         // impl
    Seed,              // impl
    Chest,             // impl
    Farmhouse,         // impl
    BallisticMissile,  // impl
    FleetDefenseShip,  // todo
    SeekingMissile,    // impl
    HypersonicMissile, // impl
    TrackingMissile,   // impl
    CruiseMissile,     // impl
    ScrapShip,         // impl
    LaserNode,         // impl
    BasicTurret,       // impl
    LaserNodeLR,       // impl
    SmartTurret,       // impl
    BlastTurret,       // todo
    LaserTurret,       // todo
    EmpZone,           // todo
}

pub enum Asset {
    // drawing assets, specifically for client side
    Simple(&'static str),                 // one image
    Partisan(&'static str, &'static str), // (friendly, enemy) for types that have different assets depending on their friendliness
    Unimpl,                               // we don't have any resources for this asset
}

impl Asset {
    pub const NOT_FOUND: &'static str = "notfound.svg";
    pub fn to_friendly(&self) -> &'static str {
        match self {
            Self::Simple(img) => img,
            Self::Partisan(friendly, _) => friendly,
            Self::Unimpl => Asset::NOT_FOUND,
        }
    }

    pub fn to_enemy(&self) -> &'static str {
        match self {
            Self::Simple(img) => img,
            Self::Partisan(_, enemy) => enemy,
            Self::Unimpl => Asset::NOT_FOUND,
        }
    }
}

pub enum Shape {
    Box(f32, f32), // width, height
    Unimpl,        // no shape data for this asset
}

impl Shape {
    pub fn to_bbox(&self) -> (f32, f32) {
        match self {
            Self::Box(w, h) => (*w, *h),
            Self::Unimpl => (50.0, 50.0),
        }
    }

    #[cfg(feature = "server")]
    pub fn to_collider(&self) -> Collider {
        match self {
            Self::Box(w, h) => Collider::rectangle(*w, *h),
            Self::Unimpl => Collider::rectangle(50.0, 50.0), // a bigass "loading failed" that kills things
        }
    }
}

pub enum SensorType {
    // what sort of things will trip this sensor?
    All,
    Some(&'static [PieceType]),
    One(PieceType),
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
            Self::Farmhouse => 30,
            Self::SeekingMissile => 10,
            Self::HypersonicMissile => 20,
            Self::TrackingMissile => 30,
            Self::CruiseMissile => 50,
            Self::LaserNode => 10,
            Self::ScrapShip => 20,
            Self::LaserNodeLR => 80,
            Self::BasicTurret => 50,
            Self::SmartTurret => 100,
            _ => 0,
        }
    }

    pub fn user_placeable(&self) -> bool {
        match self {
            Self::BasicFighter
            | Self::TieFighter
            | Self::Sniper
            | Self::CruiseMissile
            | Self::DemolitionCruiser
            | Self::Battleship
            | Self::Seed
            | Self::TrackingMissile
            | Self::Farmhouse
            | Self::BallisticMissile
            | Self::SeekingMissile
            | Self::HypersonicMissile
            | Self::ScrapShip
            | Self::LaserNode
            | Self::FleetDefenseShip
            | Self::LaserNodeLR
            | Self::BasicTurret
            | Self::SmartTurret => true, // if you want a type to be user placeable, just add it to this lil' blob.
            _ => false,
        }
    }

    pub fn user_movable(&self) -> bool {
        match self {
            Self::BasicFighter
            | Self::TieFighter
            | Self::Sniper
            | Self::CruiseMissile
            | Self::DemolitionCruiser
            | Self::Battleship
            | Self::Seed
            | Self::TrackingMissile
            | Self::BallisticMissile
            | Self::SeekingMissile
            | Self::HypersonicMissile
            | Self::ScrapShip
            | Self::FleetDefenseShip => true, // if you want a type to be movable, just add it to this lil' blob.
            _ => false,
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
            Self::LaserNode => FabLevels::defense(1),
            Self::ScrapShip => FabLevels::econ(2),
            Self::LaserNodeLR => FabLevels::defense(2),
            Self::BasicTurret => FabLevels::defense(1),
            Self::SmartTurret => FabLevels::defense(2),
            _ => FabLevels::default(),
        }
    }

    pub fn asset(&self) -> Asset {
        // specifically for the client side: get the image(s?) for the thing we're drawing
        match self {
            Self::BasicFighter => {
                Asset::Partisan("basic_fighter_friendly.svg", "basic_fighter_enemy.svg")
            }
            Self::Castle => Asset::Partisan("castle_friendly.svg", "castle_enemy.svg"),
            Self::Bullet => Asset::Simple("bullet.svg"),
            Self::TieFighter => {
                Asset::Partisan("tie_fighter_friendly.svg", "tie_fighter_enemy.svg")
            }
            Self::Sniper => Asset::Partisan("sniper_friendly.svg", "sniper_enemy.svg"),
            Self::DemolitionCruiser => Asset::Partisan(
                "demolition_cruiser_friendly.svg",
                "demolition_cruiser_enemy.svg",
            ),
            Self::Battleship => Asset::Partisan("battleship_friendly.svg", "battleship_enemy.svg"),
            Self::SmallBomb => Asset::Simple("smallbomb.svg"),
            Self::Seed => Asset::Simple("seed.svg"),
            Self::Chest => Asset::Simple("chest.svg"),
            Self::Farmhouse => Asset::Simple("farmhouse.svg"),
            Self::BallisticMissile => Asset::Partisan(
                "ballistic_missile_friendly.svg",
                "ballistic_missile_enemy.svg",
            ),
            Self::FleetDefenseShip => Asset::Unimpl,
            Self::SeekingMissile => {
                Asset::Partisan("seeking_missile_friendly.svg", "seeking_missile_enemy.svg")
            }
            Self::HypersonicMissile => Asset::Partisan(
                "hypersonic_missile_friendly.svg",
                "hypersonic_missile_enemy.svg",
            ),
            Self::TrackingMissile => Asset::Partisan(
                "tracking_missile_friendly.svg",
                "tracking_missile_enemy.svg",
            ),
            Self::CruiseMissile => {
                Asset::Partisan("cruise_missile_friendly.svg", "cruise_missile_enemy.svg")
            }
            Self::LaserNode => Asset::Simple("lasernode.svg"),
            Self::ScrapShip => Asset::Simple("scrapship.svg"),
            Self::LaserNodeLR => Asset::Simple("lasernode_lr.svg"),
            Self::BasicTurret => {
                Asset::Partisan("basic_turret_friendly.svg", "basic_turret_enemy.svg")
            }
            Self::SmartTurret => {
                Asset::Partisan("smart_turret_friendly.svg", "smart_turret_enemy.svg")
            }
            _ => Asset::Unimpl,
        }
    }

    pub fn shape(&self) -> Shape {
        match self {
            Self::BasicFighter => Shape::Box(41.0, 41.0),
            Self::Bullet => Shape::Box(10.0, 2.5),
            Self::Castle => Shape::Box(60.0, 60.0),
            Self::TieFighter => Shape::Box(40.0, 50.0),
            Self::Sniper => Shape::Box(60.0, 30.0),
            Self::DemolitionCruiser => Shape::Box(40.0, 40.0),
            Self::Battleship => Shape::Box(150.0, 200.0),
            Self::SmallBomb => Shape::Box(10.0, 10.0),
            Self::Seed => Shape::Box(7.0, 7.0),
            Self::Chest => Shape::Box(20.0, 20.0),
            Self::Farmhouse => Shape::Box(50.0, 50.0),
            Self::BallisticMissile => Shape::Box(35.0, 20.0),
            Self::FleetDefenseShip => Shape::Unimpl,
            Self::SeekingMissile => Shape::Box(35.0, 20.0),
            Self::HypersonicMissile => Shape::Box(35.0, 10.0),
            Self::TrackingMissile => Shape::Box(35.0, 17.0),
            Self::CruiseMissile => Shape::Box(35.0, 10.0),
            Self::LaserNode => Shape::Box(15.0, 15.0),
            Self::ScrapShip => Shape::Box(50.0, 50.0),
            Self::LaserNodeLR => Shape::Box(30.0, 30.0),
            Self::BasicTurret => Shape::Box(40.0, 25.0),
            Self::SmartTurret => Shape::Box(40.0, 25.0),
            _ => Shape::Unimpl,
        }
    }

    pub fn sensor(&self) -> Option<f32> {
        match self {
            Self::Farmhouse => Some(100.0),
            Self::SeekingMissile => Some(300.0),
            Self::LaserNode => Some(200.0),
            Self::LaserNodeLR => Some(600.0),
            Self::ScrapShip => Some(300.0),
            Self::BasicTurret => Some(350.0),
            Self::SmartTurret => Some(350.0),
            _ => None,
        }
    }

    pub fn field(&self) -> Option<f32> {
        // specifically for the client; returns the radius of the field to draw around this piece
        if let Some(sensor) = self.sensor() {
            return Some(sensor);
        }
        if let PieceType::Castle = *self {
            return Some(600.0);
        }
        None
    }

    pub fn show_field(&self) -> bool {
        // should the field for this piece be visible at all times?
        match self {
            Self::Farmhouse => true, // some types have overrides for fields drawn on GPU, this is just the ones drawn on CPU
            Self::BasicTurret => true,
            Self::SmartTurret => true,
            _ => false,
        }
    }

    pub fn supports_target_control(&self) -> bool {
        match self {
            Self::TrackingMissile => true,
            _ => false,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::BasicFighter => "Basic Fighter",
            Self::Bullet => "Bullet",
            Self::Castle => "Castle",
            Self::TieFighter => "Tie Fighter",
            Self::Sniper => "Sniper",
            Self::DemolitionCruiser => "Demolition Cruiser",
            Self::Battleship => "Battleship",
            Self::SmallBomb => "Small Bomb",
            Self::Seed => "Seed",
            Self::Chest => "Chest",
            Self::Farmhouse => "Farmhouse",
            Self::BallisticMissile => "Ballistic Missile",
            Self::FleetDefenseShip => "Fleet Defense Ship",
            Self::SeekingMissile => "Seeking Missile",
            Self::HypersonicMissile => "Hypersonic Missile",
            Self::TrackingMissile => "Tracking Missile",
            Self::CruiseMissile => "Cruise Missile",
            Self::LaserNode => "Small Laser Node",
            Self::ScrapShip => "Scrap Ship",
            Self::LaserNodeLR => "Large Laser Node",
            Self::BasicTurret => "Basic Turret",
            Self::SmartTurret => "Smart Turret",
            _ => "",
        }
    }

    pub fn description(&self) -> &'static str {
        // get an html description of this piece
        match self {
            Self::BasicFighter => "Slow ship that fires short-range bullets at a moderate interval",
            Self::Bullet => "A bullet!",
            Self::Castle => "A castle!",
            Self::TieFighter => "Slow ship that fires repeating bullets at a moderate interval",
            Self::Sniper => "Fast ship that fires long-range bullets at a long interval",
            Self::DemolitionCruiser => "Slow ship that fires small bombs",
            Self::Battleship => "why the hell did I make this",
            Self::SmallBomb => "A little bomb!",
            Self::Seed => "Static piece that matures into a Chest after one hundred ticks in the radius of a farm field",
            Self::Chest => "Static piece that grants whichever player kills it 20 coins",
            Self::Farmhouse => "Farmhouse that produces a small farm field in which seeds can be grown into chests",
            Self::BallisticMissile => "Cheap kinetic missile",
            Self::FleetDefenseShip => "unimplemented",
            Self::SeekingMissile => "Kinetic missile that locks onto the first enemy piece it detects",
            Self::HypersonicMissile => "Fast missile with a low-yield warhead",
            Self::TrackingMissile => "Slow missile with a low-yield warhead that can be manually onto enemy pieces",
            Self::CruiseMissile => "Slow missile with a high-yield warhead",
            Self::LaserNode => "Small laser node that creates laser walls to nearby laser nodes",
            Self::ScrapShip => "Slow, weak ship that fires laser at nearby chests to collect the $20",
            Self::LaserNodeLR => "Long range laser mnode that creates laser walls to nearby laser nodes",
            Self::BasicTurret => "Automatically swivelling turret that fires bullets at enemies in range",
            Self::SmartTurret => "Automatically swivelling turret that fires bullets at enemies in range, with much better aim",
            _ => ""
        }
    }
}
