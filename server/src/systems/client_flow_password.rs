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

pub fn client_flow_password(
    mut password_events: EventReader<ClientTriedPasswordEvent>,
    mut client_joined_event: EventWriter<ClientSuccessfullyJoinedEvent>,
    config: Res<Config>,
    channel: Query<&ClientChannel>,
    mut commands: Commands,
) {
    if let Some(p) = &config.password {
        for ClientTriedPasswordEvent(client, password) in password_events.read() {
            if p == password {
                commands
                    .entity(*client)
                    .insert(ClientAffiliation { slot: 1 });
                client_joined_event.write(ClientSuccessfullyJoinedEvent(*client));
            } else {
                if let Ok(c) = channel.get(*client) {
                    c.send(ServerMessage::Reject)
                }
            }
        }
    }
}
