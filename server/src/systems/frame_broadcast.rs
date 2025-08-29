/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// sends the current frame number and stage (and other such information) to clients every tick

use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;
use common::comms::ServerMessage;

pub fn frame_broadcast(
    broadcast: ResMut<Sender>,
    mut state: ResMut<GameState>,
    current_players: Query<&ClientAffiliation, With<ClientPlaying>>,
    config: Res<Config>,
) {
    let mut last_slot = None;
    let mut is_team_variety = false; // if there are ANY free agents, or MORE THAN 1 teams have living members, this should be true.
    let mut currently_playing = 0; // the number of total players playing
    for affiliation in current_players.iter() {
        currently_playing += 1;
        if let Some(s) = last_slot {
            if s != affiliation.slot {
                is_team_variety = true;
            }
        }
        if affiliation.slot == 1 {
            is_team_variety = true;
        } else if affiliation.slot > 1 {
            last_slot = Some(affiliation.slot);
        }
    }
    if currently_playing < config.counts.min_players as usize {
        state.playing = false;
    }
    if !is_team_variety {
        state.playing = false;
    }
    if state.playing {
        state.tick += 1;
        if state.tick > state.time_in_stage {
            state.strategy = !state.strategy;
            if state.strategy {
                state.time_in_stage = config.times.strategy_period;
            } else {
                state.time_in_stage = config.times.play_period;
            }
            state.tick = 0;
        }
    } else {
        if currently_playing >= config.counts.min_players as usize && is_team_variety {
            state.tick += 1;
        } else {
            state.tick = 0;
        }
        if state.tick > state.time_in_stage {
            state.playing = true;
        }
    }
    let _ = broadcast.send(ServerMessage::GameState {
        stage: state.get_state_enum(),
        tick_in_stage: state.tick,
        stage_duration: state.time_in_stage,
    });
}
