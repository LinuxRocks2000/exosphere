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

pub fn setup_client(
    mut events: EventReader<ClientSuccessfullyJoinedEvent>,
    mut ev_newclient: EventWriter<NewClientEvent>,
    config: Res<Config>,
    clients: Res<ClientMap>,
    broadcast: ResMut<Sender>,
    channels: Query<&ClientChannel>,
    affiliations: Query<&ClientAffiliation>,
    meta: Query<&ClientMeta>,
    mut commands: Commands,
) {
    for ClientSuccessfullyJoinedEvent(client) in events.read() {
        let client = *client;
        let id = meta.get(client).unwrap().id;
        let slot = if let Ok(a) = affiliations.get(client) {
            a.slot
        } else {
            continue;
        };
        for k in clients.keys() {
            if clients[k] != client {
                let message = ServerMessage::PlayerData {
                    id: *k,
                    nickname: meta.get(clients[k]).unwrap().nickname.clone(),
                    slot: affiliations.get(clients[k]).unwrap().slot,
                };
                channels.get(client).unwrap().send(message);
            }
        }
        channels.get(client).unwrap().send(ServerMessage::Metadata {
            id,
            board_width: config.board.width,
            board_height: config.board.height,
            slot,
        });
        if let Err(_) = broadcast.send(ServerMessage::PlayerData {
            id,
            nickname: meta.get(client).unwrap().nickname.clone(),
            slot,
        }) {
            println!("couldn't broadcast player data");
        }
        commands.entity(client).insert(ClientConnected);
        ev_newclient.write(NewClientEvent { id });
    }
}
