/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General pub(crate)lic License as pub(crate)lished by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General pub(crate)lic License for more details.

    You should have received a copy of the GNU General pub(crate)lic License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// event structures. NOT event handlers.

pub use crate::client_events::*;

use crate::components::ExplosionProperties;
use bevy::prelude::Entity;
use bevy::prelude::Event;
use bevy::prelude::Vec2;
use common::types::PieceType;
use common::PlayerId;

#[derive(Event)]
pub(crate) struct NewClientEvent {
    pub(crate) id: PlayerId,
}

#[derive(Event)]
pub(crate) struct PlaceEvent {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) a: f32,
    pub(crate) owner: PlayerId,
    pub(crate) slot: u8,
    pub(crate) tp: PieceType,
    pub(crate) free: bool, // do we need to fabber check this one? if free is set to true, fabber and territory checks are skipped
}

#[derive(Event)]
pub(crate) struct ClientKilledEvent {
    // something happened that could have killed a client
    // we'll healthcheck to see if the client actually died and update game state accordingly
    pub(crate) client: PlayerId,
}

#[derive(Event)]
pub(crate) struct ExplosionEvent {
    // an explosion was initiated!
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) props: ExplosionProperties,
}

#[derive(Event)]
pub(crate) struct PieceDestroyedEvent {
    pub(crate) piece: Entity,
    pub(crate) responsible: PlayerId, // the client responsible for this destruction (== the owner of the piece that did fatal damage)
}

#[derive(Event)]
pub(crate) struct LaserCastEvent {
    pub(crate) caster: Entity,
    pub(crate) from: Vec2,
    pub(crate) dir: Vec2,
    pub(crate) max_dist: f32,
    pub(crate) dmg: f32,
    pub(crate) exclusive: Option<Entity>, // if there's an exclusive target set, the laser will not be cast unless doing so would damage the target
}

#[derive(Event)]
pub(crate) struct PieceHarmEvent {
    pub(crate) piece: Entity,
    pub(crate) harm_amount: f32,
    pub(crate) responsible: PlayerId,
}
