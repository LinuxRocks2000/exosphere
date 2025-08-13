/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// TODO: break up into small systems rather than this behemoth
// TODO: use Observers (OnCollisionStart) rather than buffered events for some types of collision

// collision handler systems

use crate::components::*;
use crate::events::*;
use crate::ClientMap;
use avian2d::prelude::*;
use bevy::prelude::*;
use common::comms::ServerMessage;
use common::PlayerId;

pub fn handle_collisions(
    mut collision_events: EventReader<CollisionStarted>,
    mut pieces: Query<(Entity, &mut GamePiece, Option<&Bullet>, Option<&mut Seed>)>,
    mut spaceshipoids: Query<&mut Spaceshipoid>,
    explode_on_collision: Query<(Entity, &CollisionExplosion, &Transform)>,
    mut piece_destroy: EventWriter<PieceDestroyedEvent>,
    mut explosion_event: EventWriter<ExplosionEvent>,
    sensors: Query<&FieldSensor>,
) {
    for event in collision_events.read() {
        let CollisionStarted(one, two) = event;
        if let Ok((entity, explode, pos)) = explode_on_collision.get(*one) {
            piece_destroy.write(PieceDestroyedEvent {
                piece: entity,
                responsible: PlayerId::SYSTEM,
            });
            explosion_event.write(ExplosionEvent {
                x: pos.translation.x,
                y: pos.translation.y,
                props: explode.explosion,
            });
        }
        if let Ok((entity, explode, pos)) = explode_on_collision.get(*two) {
            piece_destroy.write(PieceDestroyedEvent {
                piece: entity,
                responsible: PlayerId::SYSTEM,
            });
            explosion_event.write(ExplosionEvent {
                x: pos.translation.x,
                y: pos.translation.y,
                props: explode.explosion,
            });
        }
        let mut sensor = sensors.get(*one);
        let mut sensor_is_one = true;
        if let Err(_) = sensor {
            sensor_is_one = false;
            sensor = sensors.get(*two);
        }
        if let Ok(sensor) = sensor {
            let mut sensor_owner = PlayerId::SYSTEM;
            let mut sensor_slot: u8 = 0;
            if let Ok((_, sensored_piece, _, _)) = pieces.get(sensor.attached_to) {
                sensor_owner = sensored_piece.owner;
                sensor_slot = sensored_piece.slot;
            }
            let hit_entity = if sensor_is_one {
                // the piece the sensor has reacted to
                *two
            } else {
                *one
            };
            {
                let piece = pieces.get_mut(hit_entity);
                if let Ok((entity, gamepiece, _, _)) = piece {
                    if gamepiece.owner != sensor_owner
                        && (gamepiece.slot != sensor_slot || gamepiece.slot == 1)
                    {
                        // check if the piece is enemy or not
                        if sensor.attached_to != entity {
                            // missiles can't attempt to attack themselves
                            if let Ok(mut shipoid) = spaceshipoids.get_mut(sensor.attached_to) {
                                shipoid.sensor_tripped(entity.into(), gamepiece.tp);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_destructive_collisions(
    collisions: Collisions,
    mut piece_destroy: EventWriter<PieceDestroyedEvent>,
    mut pieces: Query<(Entity, &mut GamePiece)>,
    velocities: Query<&PresolveVelocity>,
    explosions: Query<&ExplosionProperties>,
    clients: Res<ClientMap>,
    time: Res<Time<Substeps>>,
) {
    let mut one_dmg: f32 = 0.0; // damage to apply to entity 1
    let mut two_dmg: f32 = 0.0; // damage to apply to entity 2
    let mut one_killer = PlayerId::SYSTEM; // the id of the player that owned the piece that damaged the piece
    let mut two_killer = PlayerId::SYSTEM; // that is one HELL of a sentence
                                           // [tyler, several months later] that it is, laddie. that it is.
                                           // todo: defense and damage modifiers
    for collision in collisions.iter() {
        let one = if let Some(e) = collision.body1 {
            e
        } else {
            continue;
        };
        let two = if let Some(e) = collision.body2 {
            e
        } else {
            continue;
        };
        let delta_v = if let Ok(one_vel) = velocities.get(one) {
            if let Ok(two_vel) = velocities.get(two) {
                (one_vel.0 - two_vel.0).length()
            } else {
                continue;
            }
        } else {
            continue;
        };
        if delta_v >= 400.0 {
            // anything slower is nondestructive.
            // because bullets are usually moving at 400.0 and change, bullets will usually do a little over 1 damage. missiles will do quite a bit more.
            let d = (delta_v - 350.0).sqrt() / 10.0;
            one_dmg = d;
            two_dmg = d;
            if let Ok((_, piece_one)) = pieces.get(one) {
                two_killer = piece_one.owner;
            }
            if let Ok((_, piece_two)) = pieces.get(two) {
                one_killer = piece_two.owner;
            }
        }
        if one_dmg != 0.0 {
            if let Ok((entity_one, mut piece_one)) = pieces.get_mut(one) {
                piece_one.health -= one_dmg;
                if let Some(client) = clients.get(&piece_one.owner) {
                    client.send(ServerMessage::Health {
                        id: entity_one.into(),
                        health: piece_one.health / piece_one.start_health,
                    });
                }
                if piece_one.health <= 0.0 {
                    piece_destroy.write(PieceDestroyedEvent {
                        piece: entity_one,
                        responsible: one_killer,
                    });
                }
            }
        }
        if two_dmg != 0.0 {
            if let Ok((entity_two, mut piece_two)) = pieces.get_mut(two) {
                piece_two.health -= two_dmg;
                if let Some(client) = clients.get(&piece_two.owner) {
                    client.send(ServerMessage::Health {
                        id: entity_two.into(),
                        health: piece_two.health / piece_two.start_health,
                    });
                }
                if piece_two.health <= 0.0 {
                    piece_destroy.write(PieceDestroyedEvent {
                        piece: entity_two,
                        responsible: two_killer,
                    });
                }
            }
        }
        if let Ok(explosion) = explosions.get(one) {
            if let Ok((_, _)) = pieces.get(two) {
                two_dmg += explosion.damage;
            }
        }
        if let Ok(explosion) = explosions.get(two) {
            if let Ok((_, _)) = pieces.get(one) {
                one_dmg += explosion.damage;
            }
        }
    }
}
