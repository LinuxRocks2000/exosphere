/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

use crate::comms::*;
pub use crate::config::Config;
use crate::Client;
use crate::Comms;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use common::comms::Stage;
use common::PlayerId;
use std::collections::HashMap;

#[derive(Resource)]
pub struct GameState {
    pub playing: bool,
    pub io: bool,
    pub strategy: bool,
    pub tick: u16,
    pub time_in_stage: u16,
    pub currently_attached_players: u16, // the number of players CONNECTED
    pub currently_playing: u16,          // the number of players with territory
}

impl GameState {
    pub fn get_state_enum(&self) -> Stage {
        if self.playing {
            if self.strategy {
                Stage::MoveShips
            } else {
                Stage::Playing
            }
        } else {
            Stage::Waiting
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ClientMap(pub HashMap<PlayerId, Client>);

#[derive(Resource, Deref, DerefMut)] // todo: better names (or generic type arguments)
pub struct Receiver(pub crossbeam::channel::Receiver<Comms>);

#[derive(Resource, Deref, DerefMut)]
pub struct Sender(pub crossbeam::channel::Sender<ServerMessage>);

#[derive(Resource, Default)]
pub struct OneShots {
    pub board_setup: Option<SystemId>,
}
