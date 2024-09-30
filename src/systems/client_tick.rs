/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// handle messages incoming from the client

use bevy::prelude::*;
use crate::components::*;
use crate::pathfollower::*;
use bevy_rapier2d::prelude::*;
use crate::events::*;
use crate::resources::*;
use crate::Placer;
use crate::comms::*;
use crate::types::*;
use std::f32::consts::PI;
use tokio::sync::{mpsc, broadcast};
use num_traits::FromPrimitive;


pub fn client_tick(mut commands : Commands, mut pieces : Query<(Entity, &GamePiece, Option<&mut PathFollower>, Option<&Transform>, Option<&Territory>)>, mut guns : Query<&mut Gun>, mut ev_newclient : EventWriter<NewClientEvent>, place : EventWriter<PlaceEvent>, mut state : ResMut<GameState>, config : Res<GameConfig>, mut clients : ResMut<ClientMap>, mut receiver : ResMut<Receiver>, broadcast : ResMut<Sender>, mut client_killed : EventWriter<ClientKilledEvent>) {
    let mut place = Placer(place);
    // manage events from network-connected clients
    loop { // loops receiver.try_recv(), until it returns empty
        match receiver.try_recv() {
            Ok(message) => {
                match message {
                    Comms::ClientConnect(cli) => {
                        clients.insert(cli.id, cli);
                    },
                    Comms::ClientDisconnect(id) => {
                        println!("client disconnected. cleaning up!");
                        for (entity, piece, _, _, _) in pieces.iter() {
                            if piece.owner == id {
                                commands.entity(entity).despawn();
                                if let Err(_) = broadcast.send(ServerMessage::DeleteObject(entity.index())) {
                                    println!("game engine lost connection to webserver. this is probably not critical.");
                                }
                            }
                        }
                        state.currently_attached_players -= 1;
                        if clients[&id].alive {
                            state.currently_playing -= 1;
                        }
                        clients.remove(&id);
                        client_killed.send(ClientKilledEvent { client : id });
                    },
                    Comms::MessageFrom(id, msg) => {
                        let mut kill = false;
                        if clients.contains_key(&id) {
                            match msg {
                                ClientMessage::Connect(banner, _password) => { // TODO: IMPLEMENT PASSWORD
                                    let slot : u8 = if state.currently_attached_players < config.max_player_slots { 1 } else { 0 }; // todo: implement teams
                                    for k in clients.keys() {
                                        if *k != id {
                                            let message = ServerMessage::PlayerData(*k, clients[k].banner.clone(), clients[k].slot);
                                            clients[&id].send(message);
                                        }
                                    }
                                    clients.get_mut(&id).unwrap().send(ServerMessage::Metadata(id, config.width, config.height, slot));
                                    if let Err(_) = broadcast.send(ServerMessage::PlayerData(id, banner.clone(), slot)) {
                                        println!("couldn't broadcast player data");
                                    }
                                    state.currently_attached_players += 1;
                                    clients.get_mut(&id).unwrap().slot = slot;
                                    clients.get_mut(&id).unwrap().banner = banner;
                                    ev_newclient.send(NewClientEvent {id});
                                },
                                ClientMessage::PlacePiece(x, y, t) => {
                                    if let Some(t) = PieceType::from_u16(t) {
                                        if t == PieceType::Castle {
                                            if !state.playing || state.io {
                                                if clients[&id].has_placed_castle {
                                                    println!("client attempted to place an extra castle. dropping.");
                                                    kill = true;
                                                }
                                                else {
                                                    let mut is_okay = true;
                                                    for (_, _, _, transform, territory) in pieces.iter() {
                                                        if let Some(transform) = transform {
                                                            if let Some(territory) = territory {
                                                                let dx = transform.translation.x - x;
                                                                let dy = transform.translation.y - y;
                                                                let d = (dx * dx + dy * dy).sqrt();
                                                                if d < territory.radius + 600.0 { // if the territories would intersect
                                                                    is_okay = false;
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    if is_okay {
                                                        state.currently_playing += 1;
                                                        clients.get_mut(&id).unwrap().has_placed_castle = true;
                                                        clients.get_mut(&id).unwrap().alive = true;
                                                        clients.get_mut(&id).unwrap().collect(100);
                                                        let slot = clients[&id].slot;
                                                        place.castle(x, y, id, slot);
                                                        place.basic_fighter_free(x - 200.0, y, PI, id, slot);
                                                        place.basic_fighter_free(x + 200.0, y, 0.0, id, slot);
                                                        place.basic_fighter_free(x, y - 200.0, 0.0, id, slot);
                                                        place.basic_fighter_free(x, y + 200.0, 0.0, id, slot);
                                                    }
                                                }
                                            }
                                        }
                                        else if state.playing && state.strategy {
                                            let slot = clients[&id].slot;
                                            if t.user_placeable() {
                                                if clients.get_mut(&id).unwrap().charge(t.price()) {
                                                    place.p_simple(x, y, id, slot, t);
                                                }
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StrategyPointAdd(piece_id, index, x, y) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id { // TODO: FIX THIS
                                                // THIS IS REALLY BAD
                                                // REALLY REALLY REALLY BAD
                                                // WE'RE DOING LINEAR TIME LOOKUPS WHERE A CONSTANT TIME LOOKUP WOULD SUFFICE AND WELL
                                                // FIIIIIIIIIIIIIIIIIIX THISSSSSSSSSS
                                                if let Some(mut pathfollower) = pathfollower {
                                                    if piece.owner == id {
                                                        pathfollower.insert_point(index, x, y);
                                                    }
                                                    else {
                                                        println!("client attempted to move thing it doesn't own [how rude]");
                                                    }
                                                }
                                                else {
                                                    println!("attempt to move immovable object");
                                                }
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StrategyClear(piece_id) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id { // see above
                                                if let Some(mut pathfollower) = pathfollower {
                                                    if piece.owner == id {
                                                        pathfollower.clear();
                                                    }
                                                }
                                                else {
                                                    println!("whoops");
                                                }
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StrategyPointUpdate(piece_id, index, x, y) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id { // see above
                                                if let Some(mut pathfollower) = pathfollower {
                                                    if piece.owner == id {
                                                        pathfollower.update_point(index, x, y);
                                                    }
                                                }
                                                else {
                                                    println!("whoops");
                                                }
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StrategyRemove(piece_id, index) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id { // see above
                                                if let Some(mut pathfollower) = pathfollower {
                                                    if piece.owner == id {
                                                        pathfollower.remove_node(index);
                                                    }
                                                }
                                                else {
                                                    println!("whoops");
                                                }
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StrategySetEndcapRotation(piece_id, r) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id { // see above
                                                if let Some(mut pathfollower) = pathfollower {
                                                    if piece.owner == id {
                                                        pathfollower.set_endcap_rotation(r);
                                                    }
                                                }
                                                else {
                                                    println!("whoops");
                                                }
                                            }
                                        }
                                    }
                                },
                                ClientMessage::StrategyTargetAdd(piece_id, t) => {
                                    if state.playing && state.strategy {
                                        let mut ent : Option<Entity> = None;
                                        for (entity, _, _, _, _) in pieces.iter() {
                                            if entity.index() == t {
                                                ent = Some(entity);
                                                break;
                                            }
                                        }
                                        if let Some(target) = ent {
                                            for (entity, piece, pathfollower, _, _) in pieces.iter_mut() {
                                                if entity.index() == piece_id {
                                                    if let Some(mut pathfollower) = pathfollower {
                                                        if piece.owner == id {
                                                            let pos = pathfollower.len();
                                                            pathfollower.insert_target(pos, target);
                                                        }
                                                    }
                                                    else {
                                                        println!("whoops");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                ClientMessage::GunState(piece_id, gun_state) => {
                                    if state.playing && state.strategy {
                                        for (entity, piece, _, _, _) in pieces.iter_mut() {
                                            if entity.index() == piece_id {
                                                if piece.owner == id {
                                                    if let Ok(mut gun) = guns.get_mut(entity) {
                                                        if gun_state == 0 {
                                                            gun.enabled = false;
                                                        }
                                                        else {
                                                            gun.enabled = true;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    println!("error: client sent unimplemented frame! dropping client.");
                                    kill = true;
                                }
                            }
                        }
                        else {
                            println!("error: received message from client {}, which does not exist", id);
                        }
                        if kill {
                            clients.remove(&id);
                        }
                    }
                }
            },
            Err(mpsc::error::TryRecvError::Empty) => {
                break;
            }
            _ => {
                println!("ERROR OCCURRED! TAMERE!");
            }
        }
    }
}