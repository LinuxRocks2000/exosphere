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

pub fn client_connection(
    mut events: EventReader<ClientConnectEvent>,
    mut commands: Commands,
    config: Res<Config>,
    channels: Query<&ClientChannel>,
    cl: Query<&Client>,
    mut success: EventWriter<ClientSuccessfullyJoinedEvent>,
) {
    for ClientConnectEvent(client, nickname) in events.read() {
        let id = cl.get(*client).unwrap().id;
        commands.entity(*client).insert(ClientMeta {
            id,
            nickname: nickname.clone(),
        });
        if let Some(teams) = &config.teams {
        } else if let Some(_) = config.password {
            channels
                .get(*client)
                .unwrap()
                .send(ServerMessage::PasswordChallenge);
        } else {
            commands
                .entity(*client)
                .insert(ClientAffiliation { slot: 1 });
            success.write(ClientSuccessfullyJoinedEvent(*client));
        }
    }
}
