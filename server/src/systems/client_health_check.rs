/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// checks on the health of every client and broadcasts lose conditions if necessary

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use common::comms::ServerMessage;

pub fn client_health_check(
    mut commands: Commands,
    mut events: EventReader<ClientKilledEvent>,
    mut piece_kill: EventWriter<PieceDestroyedEvent>,
    clients: Res<ClientMap>,
    pieces: Query<(Option<&Territory>, &GamePiece, Entity)>,
    channels: Query<&ClientChannel>,
    mut lose_event: EventWriter<ClientLostEvent>,
) {
    // checks:
    // * if the client is still present (if the client disconnected, it's dead by default!), exit early
    // * if the client has any remaining Territory, it's not dead, false alarm
    // if we determined that the client is in fact dead, send a Lose message and update the state accordingly.
    for ev in events.read() {
        if let Some(client) = clients.get(&ev.client) {
            let client = *client;
            // if the client's already disconnected, we can't exactly tell them they lost
            let mut has_territory = false;
            for (territory, piece, _) in pieces.iter() {
                if territory.is_some() && piece.owner == ev.client {
                    has_territory = true;
                }
            }
            if !has_territory {
                lose_event.write(ClientLostEvent);
                channels.get(client).unwrap().send(ServerMessage::YouLose);
                commands.entity(client).try_remove::<ClientPlaying>();
            }
        }
        for (_, piece, entity) in pieces.iter() {
            if piece.owner == ev.client {
                piece_kill.write(PieceDestroyedEvent {
                    piece: entity.into(),
                    responsible: ev.client,
                });
            }
        }
    }
}
