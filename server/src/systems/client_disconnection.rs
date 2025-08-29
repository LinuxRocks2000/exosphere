/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use common::comms::ServerMessage;

pub fn client_disconnection(
    mut events: EventReader<ClientKilledEvent>,
    mut commands: Commands,
    pieces: Query<(Entity, &mut GamePiece)>,
    broadcast: Res<Sender>,
    mut clients: ResMut<ClientMap>,
) {
    for event in events.read() {
        for (entity, piece) in pieces.iter() {
            if piece.owner == event.client {
                commands.entity(entity).despawn();
                if let Err(_) = broadcast.send(ServerMessage::DeleteObject { id: entity.into() }) {
                    println!(
                        "game engine lost connection to webserver. this is probably not critical."
                    );
                }
            }
        }
        if let Some(cl) = clients.get(&event.client) {
            commands.entity(*cl).despawn();
        }
        clients.remove(&event.client);
    }
}
