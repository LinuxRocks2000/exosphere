/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// handle pieces being hurt

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use common::comms::ServerMessage;

pub fn piece_harm(
    mut hurt: EventReader<PieceHarmEvent>,
    mut destroy: EventWriter<PieceDestroyedEvent>,
    mut pieces: Query<&mut GamePiece>,
    clients: Res<ClientMap>,
    channels: Query<&ClientChannel>,
) {
    for event in hurt.read() {
        if let Ok(mut piece) = pieces.get_mut(event.piece) {
            piece.health -= event.harm_amount;
            if let Some(client) = clients.get(&piece.owner) {
                channels.get(*client).unwrap().send(ServerMessage::Health {
                    id: event.piece.into(),
                    health: piece.health / piece.start_health,
                });
            }
            if piece.health <= 0.0 {
                destroy.write(PieceDestroyedEvent {
                    piece: event.piece,
                    responsible: event.responsible,
                });
            }
        }
    }
}
