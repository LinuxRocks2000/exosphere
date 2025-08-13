/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// handle messages incoming from the client

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::Comms;
use crate::Placer;
use bevy::prelude::*;
use common::comms::*;
use common::types::*;
use std::f32::consts::PI;

pub fn client_tick(
    mut commands: Commands,
    mut pieces: Query<(
        Entity,
        &GamePiece,
        Option<&mut Spaceshipoid>,
        Option<&Transform>,
        Option<&Territory>,
    )>,
    mut guns: Query<&mut Gun>,
    mut ev_newclient: EventWriter<NewClientEvent>,
    place: EventWriter<PlaceEvent>,
    mut state: ResMut<GameState>,
    config: Res<GameConfig>,
    mut clients: ResMut<ClientMap>,
    receiver: ResMut<Receiver>,
    broadcast: ResMut<Sender>,
    mut client_killed: EventWriter<ClientKilledEvent>,
) {
    let mut place = Placer(place);
    // manage events from network-connected clients
    loop {
        // loops receiver.try_recv(), until it returns empty
        match receiver.try_recv() {
            Ok(message) => {
                match message {
                    Comms::ClientConnect(cli) => {
                        clients.insert(cli.id, cli);
                    }
                    Comms::ClientDisconnect(id) => {
                        println!("client disconnected. cleaning up!");
                        for (entity, piece, _, _, _) in pieces.iter() {
                            if piece.owner == id {
                                commands.entity(entity).despawn();
                                if let Err(_) = broadcast
                                    .send(ServerMessage::DeleteObject { id: entity.into() })
                                {
                                    println!("game engine lost connection to webserver. this is probably not critical.");
                                }
                            }
                        }
                        if clients[&id].connected {
                            state.currently_attached_players -= 1;
                        }
                        if clients[&id].alive {
                            state.currently_playing -= 1;
                        }
                        clients.remove(&id);
                        client_killed.write(ClientKilledEvent { client: id });
                    }
                    Comms::MessageFrom(id, msg) => {
                        let mut kill = false;
                        if clients.contains_key(&id) {
                            match msg {
                                ClientMessage::Connect {
                                    nickname,
                                    password: _password,
                                } => {
                                    // TODO: IMPLEMENT PASSWORD
                                    let slot: u8 = if state.currently_attached_players
                                        < config.max_player_slots
                                    {
                                        1
                                    } else {
                                        0
                                    }; // todo: implement teams
                                    for k in clients.keys() {
                                        if *k != id {
                                            let message = ServerMessage::PlayerData {
                                                id: *k,
                                                nickname: clients[k].nickname.clone(),
                                                slot: clients[k].slot,
                                            };
                                            clients[&id].send(message);
                                        }
                                    }
                                    clients.get_mut(&id).unwrap().send(ServerMessage::Metadata {
                                        id,
                                        board_width: config.width,
                                        board_height: config.height,
                                        slot,
                                    });
                                    if let Err(_) = broadcast.send(ServerMessage::PlayerData {
                                        id,
                                        nickname: nickname.clone(),
                                        slot,
                                    }) {
                                        println!("couldn't broadcast player data");
                                    }
                                    state.currently_attached_players += 1;
                                    clients.get_mut(&id).unwrap().slot = slot;
                                    clients.get_mut(&id).unwrap().nickname = nickname;
                                    clients.get_mut(&id).unwrap().connected = true;
                                    ev_newclient.write(NewClientEvent { id });
                                }
                                ClientMessage::PlacePiece { x, y, tp } => {
                                    if tp == PieceType::Castle {
                                        if !state.playing || state.io {
                                            if clients[&id].has_placed_castle {
                                                println!("client attempted to place an extra castle. dropping.");
                                                kill = true;
                                            } else {
                                                let mut is_okay = true;
                                                for (_, _, _, transform, territory) in pieces.iter()
                                                {
                                                    if let Some(transform) = transform {
                                                        if let Some(territory) = territory {
                                                            let dx = transform.translation.x - x;
                                                            let dy = transform.translation.y - y;
                                                            let d = (dx * dx + dy * dy).sqrt();
                                                            if d < territory.radius + 600.0 {
                                                                // if the territories would intersect
                                                                is_okay = false;
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                                if is_okay {
                                                    state.currently_playing += 1;
                                                    clients
                                                        .get_mut(&id)
                                                        .unwrap()
                                                        .has_placed_castle = true;
                                                    clients.get_mut(&id).unwrap().alive = true;
                                                    clients.get_mut(&id).unwrap().collect(10000);
                                                    let slot = clients[&id].slot;
                                                    place.castle(x, y, id, slot);
                                                    place.basic_fighter_free(
                                                        x - 200.0,
                                                        y,
                                                        PI,
                                                        id,
                                                        slot,
                                                    );
                                                    place.basic_fighter_free(
                                                        x + 200.0,
                                                        y,
                                                        0.0,
                                                        id,
                                                        slot,
                                                    );
                                                    place.basic_fighter_free(
                                                        x,
                                                        y - 200.0,
                                                        0.0,
                                                        id,
                                                        slot,
                                                    );
                                                    place.basic_fighter_free(
                                                        x,
                                                        y + 200.0,
                                                        0.0,
                                                        id,
                                                        slot,
                                                    );
                                                }
                                            }
                                        }
                                    } else if state.playing && state.strategy {
                                        let slot = clients[&id].slot;
                                        if tp.user_placeable() {
                                            if clients.get_mut(&id).unwrap().charge(tp.price()) {
                                                place.p_simple(x, y, id, slot, tp);
                                            }
                                        }
                                    }
                                }
                                ClientMessage::StrategyInsert { piece, index, node } => {
                                    if state.playing && state.strategy {
                                        if let Ok((_, piece, shipoid, _, _)) =
                                            pieces.get_mut(piece.into())
                                        {
                                            if let Some(mut shipoid) = shipoid {
                                                if piece.owner == id {
                                                    shipoid.pathfollower.insert_node(index, node);
                                                } else {
                                                    println!("client attempted to move thing it doesn't own [how rude]");
                                                }
                                            } else {
                                                println!("attempt to move immovable object");
                                            }
                                        }
                                    }
                                }
                                ClientMessage::StrategyClear { piece: piece_id } => {
                                    if state.playing && state.strategy {
                                        if let Ok((_, piece, shipoid, _, _)) =
                                            pieces.get_mut(piece_id.into())
                                        {
                                            if let Some(mut shipoid) = shipoid {
                                                if piece.owner == id {
                                                    shipoid.pathfollower.clear();
                                                }
                                            } else {
                                                println!("whoops");
                                            }
                                        }
                                    }
                                }
                                ClientMessage::StrategySet {
                                    piece: piece_id,
                                    index,
                                    node,
                                } => {
                                    if state.playing && state.strategy {
                                        if let Ok((_, piece, shipoid, _, _)) =
                                            pieces.get_mut(piece_id.into())
                                        {
                                            if let Some(mut shipoid) = shipoid {
                                                if piece.owner == id {
                                                    shipoid.pathfollower.update_node(index, node);
                                                }
                                            } else {
                                                println!("whoops");
                                            }
                                        }
                                    }
                                }
                                ClientMessage::StrategyDelete {
                                    piece: piece_id,
                                    index,
                                } => {
                                    if state.playing && state.strategy {
                                        if let Ok((_, piece, shipoid, _, _)) =
                                            pieces.get_mut(piece_id.into())
                                        {
                                            if let Some(mut shipoid) = shipoid {
                                                if piece.owner == id {
                                                    shipoid.pathfollower.remove_node(index);
                                                }
                                            } else {
                                                println!("whoops");
                                            }
                                        }
                                    }
                                }
                                ClientMessage::GunState {
                                    piece: piece_id,
                                    enabled,
                                } => {
                                    if state.playing && state.strategy {
                                        if let Ok((_, piece, _, _, _)) =
                                            pieces.get_mut(piece_id.into())
                                        {
                                            if piece.owner == id {
                                                if let Ok(mut gun) = guns.get_mut(piece_id.into()) {
                                                    gun.enabled = enabled;
                                                }
                                            }
                                        }
                                    }
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
                }
            }
            Err(crossbeam::channel::TryRecvError::Empty) => {
                break;
            }
            Err(e) => {
                panic!("error: {:?}", e);
            }
        }
    }
}
