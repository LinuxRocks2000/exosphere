/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

use bevy::prelude::*;
use common::comms::ServerMessage;
use common::PlayerId;

#[derive(Component)]
pub struct ClientMoney {
    pub money: u32,
}

// markers applied to clients, this makes querying faster and allows us to structure systems in a nicer way
#[derive(Component)]
pub struct ClientPlaying; // authentication flow complete, the client is an active player
#[derive(Component)]
pub struct ClientHasPlacedCastle; // the client has placed its castle!
#[derive(Component)]
pub struct ClientConnected; // the client is connected but not necessarily authenticated or play ing
#[derive(Component)]
pub struct ClientPasswordChallenged; // the client has been sent a password challenge and should respond with a password
#[derive(Component)]
pub struct ClientTeamChallenged; // the client has been sent a team challenge and should respond with a slot number and password
#[derive(Component)]
pub struct ClientSpectating; // the client is not playing: it will receive updates but cannot send messages in chat or anything

#[derive(Component)] // the main client component. SHOULD BE EMPTY! IT'S A MARKER! if there's stuff in here it's because I'm not finished changing how logic works
pub struct Client {
    pub id: PlayerId, // don't use this for anything else!
}

#[derive(Component)]
pub struct ClientMeta {
    pub nickname: String,
    pub id: PlayerId,
}
#[derive(Component)]
pub struct ClientAffiliation {
    pub slot: u8,
}
#[derive(Component)]
pub struct ClientChannel {
    pub id: PlayerId,
    pub channel: crossbeam::channel::Sender<(PlayerId, ServerMessage)>,
}

impl ClientChannel {
    pub fn send(&self, msg: ServerMessage) {
        if let Err(_) = self.channel.try_send((self.id, msg)) {
            println!("failed to send message on channel");
        }
    }
}
