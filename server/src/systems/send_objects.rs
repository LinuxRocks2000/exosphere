/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// sends objects to new clients

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use common::comms::ServerMessage;

pub fn send_objects(
    mut events: EventReader<NewClientEvent>,
    mut clients: ResMut<ClientMap>,
    objects: Query<(
        Entity,
        &GamePiece,
        &Transform,
        Option<&Territory>,
        Option<&Fabber>,
    )>,
    channel: Query<&ClientChannel>,
) {
    for ev in events.read() {
        if let Some(client) = clients.get_mut(&ev.id) {
            for (entity, piece, transform, territory, fabber) in objects.iter() {
                let chan = channel.get(*client).unwrap();

                chan.send(ServerMessage::ObjectCreate {
                    x: transform.translation.x,
                    y: transform.translation.y,
                    a: transform.rotation.to_euler(EulerRot::ZYX).0,
                    owner: piece.owner,
                    id: entity.into(),
                    tp: piece.tp,
                });
                if let Some(territory) = territory {
                    chan.send(ServerMessage::Territory {
                        id: entity.into(),
                        radius: territory.radius,
                    });
                }
                if let Some(fabber) = fabber {
                    chan.send(ServerMessage::Fabber {
                        id: entity.into(),
                        radius: fabber.radius,
                    });
                }
            }
        }
    }
}
