/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

use crate::comms::*;
use crate::Client;
use crate::Comms;
use bevy::prelude::*;
use common::comms::Stage;
use common::PlayerId;
use std::collections::HashMap;

// todo: break up GameConfig and GameState into smaller structs for better parallelism
#[derive(Resource)]
pub struct GameConfig {
    pub width: f32,
    pub height: f32,
    pub wait_period: u16,     // time it waits before the game starts
    pub play_period: u16,     // length of a play period
    pub strategy_period: u16, // length of a strategy period
    pub max_player_slots: u16,
    pub min_player_slots: u16,
}

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
