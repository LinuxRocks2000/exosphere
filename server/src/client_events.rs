/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

use bevy::prelude::*;
use common::comms::*;
use common::types::*;
use common::PieceId;

#[derive(Event)]
pub struct ClientPlaceEvent {
    pub x: f32,
    pub y: f32,
    pub tp: PieceType,
    pub client: Entity,
}

#[derive(Event)]
pub struct ClientCollectEvent {
    pub client: Entity,
    pub amount: i32,
}

// authentication-level events
#[derive(Event)]
pub struct ClientConnectEvent(pub Entity, pub String); // a client tried to connect. we may need to reject or challenge
#[derive(Event)]
pub struct ClientTriedPasswordEvent(pub Entity, pub String); // a client tried to use the password
#[derive(Event)]
pub struct ClientTriedTeamConnectEvent(pub Entity, pub u8, pub String); // a client tried to add itself to a team
#[derive(Event)]
pub struct ClientRequestedSpectateEvent(pub Entity); // a client now wants to spectate
                                                     // disconnects aren't handled in an event as the logic is pretty light and is better to run immediately.
#[derive(Event)]
pub struct ClientSuccessfullyJoinedEvent(pub Entity); // whatever the cause, a client now needs to receive its metadata frame
                                                      // and all of the object data on the board
#[derive(Event)]
pub struct ClientSpecialObjectEvent(pub Entity, pub PieceId, pub ObjectSpecialPropertySet); // set some special property. this is for stuff like gun states and constructor positioning

#[derive(Event)]
pub struct StrategyPathModifiedEvent(pub Entity, pub StrategyPathModification);
