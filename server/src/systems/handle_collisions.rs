/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// collision handler system

use crate::events::*;
use crate::components::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::ClientMap;
use common::PlayerId;
use common::comms::ServerMessage;


pub fn handle_collisions(mut collision_events: EventReader<CollisionEvent>,
    mut pieces : Query<(Entity, &mut GamePiece, Option<&Bullet>, Option<&mut Seed>)>,
    mut spaceshipoids : Query<&mut Spaceshipoid>,
    farms : Query<&Farmhouse>,
    explode_on_collision : Query<(Entity, &CollisionExplosion, &Transform)>,
    explosions : Query<&ExplosionProperties>,
    velocities : Query<&Velocity, Without<StaticWall>>, // we use this to calculate the relative VELOCITY (NOT collision energy - it's physically inaccurate, but idc) and then the damage they incur.
    // this means at low speeds you can safely puuuuush, but at high speeds you get destroyed
    mut piece_destroy : EventWriter<PieceDestroyedEvent>,
    mut explosion_event : EventWriter<ExplosionEvent>,
    sensors : Query<&FieldSensor>,
    lasernodes : Query<&LaserNode>,
    mut scrapships : Query<&mut ScrapShip>,
    chests : Query<&Chest>,
    clients : Res<ClientMap>) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(one, two, _) = event {
            if let Ok((entity, explode, pos)) = explode_on_collision.get(*one) {
                piece_destroy.send(PieceDestroyedEvent { piece : entity, responsible : PlayerId::SYSTEM });
                explosion_event.send(ExplosionEvent {
                    x : pos.translation.x,
                    y : pos.translation.y,
                    props : explode.explosion
                });
            }
            if let Ok((entity, explode, pos)) = explode_on_collision.get(*two) {
                piece_destroy.send(PieceDestroyedEvent { piece : entity, responsible : PlayerId::SYSTEM });
                explosion_event.send(ExplosionEvent {
                    x : pos.translation.x,
                    y : pos.translation.y,
                    props : explode.explosion
                });
            }
            let mut one_dmg : f32 = 0.0; // damage to apply to entity 1
            let mut two_dmg : f32 = 0.0; // damage to apply to entity 2
            let mut one_killer = PlayerId::SYSTEM; // the id of the player that owned the piece that damaged the piece
            let mut two_killer = PlayerId::SYSTEM; // that is one HELL of a sentence
            // [tyler, several months later] that it is, laddie. that it is.
            // todo: defense and damage modifiers
            if let Ok(v1) = velocities.get(*one) {
                if let Ok(v2) = velocities.get(*two) {
                    let r_vel = (v1.linvel - v2.linvel).length();
                    if r_vel >= 400.0 { // anything slower is nondestructive.
                        // because bullets are usually moving at 400.0 and change, bullets will usually do a little over 1 damage. missiles will do quite a bit more.
                        let d = (r_vel - 350.0).sqrt() / 10.0;
                        one_dmg = d;
                        two_dmg = d;
                        if let Ok((_, piece_one, _, _)) = pieces.get(*one) {
                            two_killer = piece_one.owner;
                        }
                        if let Ok((_, piece_two, _, _)) = pieces.get(*two) {
                            one_killer = piece_two.owner;
                        }
                    }
                }
            }
            if let Ok(explosion) = explosions.get(*one) {
                if let Ok((_, _, _, _)) = pieces.get(*two) {
                    two_dmg += explosion.damage;
                }
            }
            if let Ok(explosion) = explosions.get(*two) {
                if let Ok((_, _, _, _)) = pieces.get(*one) {
                    one_dmg += explosion.damage;
                }
            }
            if one_dmg != 0.0 {
                if let Ok((entity_one, mut piece_one, _, _)) = pieces.get_mut(*one) {
                    piece_one.health -= one_dmg;
                    if let Some(client) = clients.get(&piece_one.owner) {
                        client.send(ServerMessage::Health { id : entity_one.into(), health : piece_one.health / piece_one.start_health });
                    }
                    if piece_one.health <= 0.0 {
                        piece_destroy.send(PieceDestroyedEvent { piece : entity_one, responsible : one_killer });
                    }
                }
            }
            if two_dmg != 0.0 {
                if let Ok((entity_two, mut piece_two, _, _)) = pieces.get_mut(*two) {
                    piece_two.health -= two_dmg;
                    if let Some(client) = clients.get(&piece_two.owner) {
                        client.send(ServerMessage::Health { id : entity_two.into(), health : piece_two.health / piece_two.start_health });
                    }
                    if piece_two.health <= 0.0 {
                        piece_destroy.send(PieceDestroyedEvent { piece : entity_two, responsible : two_killer });
                    }
                }
            }
        }
        let (one, two) = match event { CollisionEvent::Started(one, two, _) => (one, two), CollisionEvent::Stopped(one, two, _) => (one, two) };
        let mut sensor = sensors.get(*one);
        let mut sensor_is_one = true;
        if let Err(_) = sensor {
            sensor_is_one = false;
            sensor = sensors.get(*two);
        }
        if let Ok(sensor) = sensor {
            let mut sensor_owner = PlayerId::SYSTEM;
            let mut sensor_slot : u8 = 0;
            if let Ok((_, sensored_piece, _, _)) = pieces.get(sensor.attached_to) {
                sensor_owner = sensored_piece.owner;
                sensor_slot = sensored_piece.slot;
            }
            let hit_entity = if sensor_is_one { // the piece the sensor has reacted to
                *two
            } else {
                *one
            };
            {
                let piece = pieces.get_mut(hit_entity);
                if let Ok((entity, gamepiece, _, seed)) = piece { // handle seeds
                    if let Some(mut seed) = seed {
                        if let Ok(_) = farms.get(sensor.attached_to) {
                            if let CollisionEvent::Started(_, _, _) = event {
                                seed.growing = true;
                            }
                            else {
                                seed.growing = false;
                            }
                        }
                    }
                    if gamepiece.owner != sensor_owner && (gamepiece.slot != sensor_slot || gamepiece.slot == 1) { // check if the piece is enemy or not
                        if sensor.attached_to != entity { // missiles can't attempt to attack themselves
                            if let Ok(mut shipoid) = spaceshipoids.get_mut(sensor.attached_to) {       
                                shipoid.sensor_tripped(entity.into());
                            }
                        }
                    }
                }
            }
            'lasers : {
                if hit_entity != sensor.attached_to {
                    if let Err(_) = lasernodes.get(hit_entity) {
                        break 'lasers;
                    }
                    if let Ok(node) = lasernodes.get(sensor.attached_to) {
                        if let CollisionEvent::Started(_, _, _) = event {
                            node.connect(hit_entity);
                        }
                        else {
                            node.disconnect(hit_entity);
                        }
                        node.recalculate(&lasernodes);
                    }
                    else {
                        break 'lasers;
                    }
                    if let Ok(node) = lasernodes.get(hit_entity) {
                        if let CollisionEvent::Started(_, _, _) = event {
                            node.connect(sensor.attached_to);
                        }
                        else {
                            node.disconnect(sensor.attached_to);
                        }
                        node.recalculate(&lasernodes);
                    }
                }
            }
            {
                if let Ok(mut ss) = scrapships.get_mut(sensor.attached_to) {
                    if let Ok(_) = chests.get(hit_entity) {
                        if let CollisionEvent::Started(_, _, _) = event {
                            ss.enter(hit_entity);
                        }
                        else {
                            ss.leave(hit_entity);
                        }
                    }
                }
            }
        }
    }
}
