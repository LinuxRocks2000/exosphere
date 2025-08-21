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
use crate::Comms;
use bevy::prelude::*;
use common::comms::*;

pub fn client_tick(
    mut commands: Commands,
    mut clients: ResMut<ClientMap>,
    receiver: ResMut<Receiver>,
    mut client_killed_event: EventWriter<ClientKilledEvent>,
    mut client_placed_event: EventWriter<ClientPlaceEvent>,
    mut client_connected_event: EventWriter<ClientConnectEvent>,
    mut client_password_event: EventWriter<ClientTriedPasswordEvent>,
    mut strategy_path_modified_event: EventWriter<StrategyPathModifiedEvent>,
    mut client_special_event: EventWriter<ClientSpecialObjectEvent>,
) {
    // manage events from network-connected clients. this is just a dispatch controller; it aims to be light so the next steps can be massively
    // parallellized.
    loop {
        // loops receiver.try_recv(), until it returns empty
        match receiver.try_recv() {
            Ok(message) => match message {
                Comms::ClientConnect(id, channel) => {
                    let thing = commands.spawn((ClientChannel { id, channel }, Client { id }));
                    clients.insert(id, thing.id());
                }
                Comms::ClientDisconnect(id) => {
                    clients.remove(&id);
                    client_killed_event.write(ClientKilledEvent { client: id });
                }
                Comms::MessageFrom(id, msg) => {
                    let mut kill = false;
                    if let Some(client) = clients.get(&id) {
                        let client = *client;
                        match msg {
                            ClientMessage::Connect { nickname } => {
                                client_connected_event.write(ClientConnectEvent(client, nickname));
                            }
                            ClientMessage::TryPassword { password } => {
                                client_password_event
                                    .write(ClientTriedPasswordEvent(clients[&id], password));
                            }
                            ClientMessage::PlacePiece { x, y, tp } => {
                                client_placed_event.write(ClientPlaceEvent { x, y, tp, client });
                            }
                            ClientMessage::Strategy { evt } => {
                                strategy_path_modified_event
                                    .write(StrategyPathModifiedEvent(clients[&id], evt));
                            }
                            ClientMessage::Special { id: piece_id, evt } => {
                                client_special_event.write(ClientSpecialObjectEvent(
                                    clients[&id],
                                    piece_id,
                                    evt,
                                ));
                            }
                            _ => {
                                println!(
                                    "error: client sent unimplemented frame! dropping client."
                                );
                                kill = true;
                            }
                        }
                    } else {
                        println!(
                            "error: received message from client {:?}, which does not exist",
                            id
                        );
                    }
                    if kill {
                        clients.remove(&id);
                    }
                }
            },
            Err(crossbeam::channel::TryRecvError::Empty) => {
                break;
            }
            Err(e) => {
                panic!("error: {:?}", e);
            }
        }
    }
}
