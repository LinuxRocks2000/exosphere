/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// constants

pub const UPDATE_RATE : u64 = 30; // 30hz by default
pub const FRAME_TIME : std::time::Duration = std::time::Duration::from_millis(1000 / UPDATE_RATE); // milliseconds per frame

pub const MAX_FRAME_SIZE : usize = 1024; // maximum size of an incoming websocket frame

pub const VERSION : u8 = 0; // bump this up every time a major change is made

pub const BASIC_FIGHTER_TYPE_NUM : u16 = 0;
pub const CASTLE_TYPE_NUM : u16 = 1;
pub const BULLET_TYPE_NUM : u16 = 2;
pub const TIE_FIGHTER_TYPE_NUM : u16 = 3;
pub const SNIPER_TYPE_NUM : u16 = 4;
pub const DEMOLITION_CRUISER_TYPE_NUM : u16 = 5;
pub const BATTLESHIP_TYPE_NUM : u16 = 6;
pub const SMALL_BOMB_TYPE_NUM : u16 = 7;
pub const SEED_TYPE_NUM : u16 = 8;
pub const CHEST_TYPE_NUM : u16 = 9;
pub const FARMHOUSE_TYPE_NUM : u16 = 10;
pub const BALLISTIC_MISSILE_TYPE_NUM : u16 = 11;