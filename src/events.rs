/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// event structures. NOT event handlers.

use bevy::prelude::Event;
use bevy::prelude::Entity;
use crate::PlaceType;
use crate::ExplosionProperties;

#[derive(Event)]
pub struct NewClientEvent {
    pub id : u64
}


#[derive(Event)]
pub struct PlaceEvent {
    pub x : f32,
    pub y : f32,
    pub a : f32,
    pub owner : u64,
    pub slot : u8,
    pub tp : PlaceType,
    pub free : bool // do we need to fabber check this one? if free is set to true, fabber and territory checks are skipped
}


#[derive(Event)]
pub struct ClientKilledEvent { // something happened that could have killed a client
    // we'll healthcheck to see if the client actually died and update game state accordingly
    pub client : u64
}


#[derive(Event)]
pub struct ExplosionEvent { // an explosion was initiated!
    pub x : f32,
    pub y : f32,
    pub props : ExplosionProperties
}


#[derive(Event)]
pub struct PieceDestroyedEvent {
    pub piece : Entity
}