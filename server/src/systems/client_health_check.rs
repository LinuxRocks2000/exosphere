/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// checks on the health of every client and broadcasts win/lose conditions if necessary

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::EmptyWorld;
use bevy::prelude::*;
use common::comms::ServerMessage;
use common::PlayerId;

pub fn client_health_check(
    mut commands: Commands,
    mut events: EventReader<ClientKilledEvent>,
    mut piece_kill: EventWriter<PieceDestroyedEvent>,
    mut clients: ResMut<ClientMap>,
    pieces: Query<(Option<&Territory>, &GamePiece, Entity)>,
    mut state: ResMut<GameState>,
    config: Res<GameConfig>,
) {
    // checks:
    // * if the client is still present (if the client disconnected, it's dead by default!), exit early
    // * if the client has any remaining Territory, it's not dead, false alarm
    // if we determined that the client is in fact dead, send a Lose message and update the state accordingly.
    // At the end, if there is 1 or 0 players left, send a Win broadcast as appropriate and reset the state for the next game.
    let mut did_something = false;
    for ev in events.read() {
        if clients.contains_key(&ev.client) {
            // if the client's already disconnected, we can't exactly tell them they lost
            let mut has_territory = false;
            for (territory, piece, _) in pieces.iter() {
                if territory.is_some() && piece.owner == ev.client {
                    has_territory = true;
                }
            }
            if !has_territory {
                state.currently_playing -= 1;
                clients[&ev.client].send(ServerMessage::YouLose);
                clients.get_mut(&ev.client).unwrap().alive = false;
                if clients[&ev.client].id != PlayerId::SYSTEM {
                    for (_, piece, entity) in pieces.iter() {
                        if piece.owner == clients[&ev.client].id {
                            piece_kill.send(PieceDestroyedEvent {
                                piece: entity.into(),
                                responsible: ev.client,
                            });
                        }
                    }
                }
            }
        } else {
            for (_, piece, entity) in pieces.iter() {
                if piece.owner == ev.client {
                    piece_kill.send(PieceDestroyedEvent {
                        piece: entity.into(),
                        responsible: ev.client,
                    });
                }
            }
        }
        did_something = true;
    }
    if !state.io && did_something {
        // only if we made a change does it make sense to update the state here
        if state.playing && state.currently_playing < 2 {
            if state.currently_playing == 1 {
                let mut winid = PlayerId::SYSTEM;
                for (id, client) in clients.iter() {
                    if client.alive {
                        winid = *id;
                        break;
                    }
                }
                for (_, client) in clients.iter() {
                    client.send(ServerMessage::Winner { id: winid });
                    client.send(ServerMessage::Disconnect);
                }
            }
            state.playing = false;
            state.strategy = false;
            state.tick = 0;
            state.time_in_stage = config.wait_period;
            state.currently_playing = 0;
            commands.add(EmptyWorld {});
        }
        if state.currently_playing < config.min_player_slots {
            state.playing = false;
            state.tick = 0;
            state.time_in_stage = config.wait_period;
            state.strategy = false;
        }
    }
}
