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
use avian2d::prelude::*;
use bevy::prelude::*;
use common::comms::*;
use common::types::PieceType;

pub fn setup_sensor_observers(
    thing: &PieceType,
    attach: Entity,
    mut sensor: bevy::prelude::EntityCommands<'_>,
) {
    match thing {
        PieceType::Farmhouse => {
            sensor.observe(
                |trigger: Trigger<OnCollisionStart>, mut seeds_query: Query<&mut Seed>| {
                    let seed = trigger.target();
                    if let Ok(mut seed) = seeds_query.get_mut(seed) {
                        seed.growing = true;
                    }
                },
            );
            sensor.observe(
                |trigger: Trigger<OnCollisionEnd>, mut seeds_query: Query<&mut Seed>| {
                    let seed = trigger.target();
                    if let Ok(mut seed) = seeds_query.get_mut(seed) {
                        seed.growing = false;
                    }
                },
            );
        }
        PieceType::LaserNode | PieceType::LaserNodeLR => {
            sensor.observe(
                move |trigger: Trigger<OnCollisionStart>, lasernodes: Query<&LaserNode>| {
                    if trigger.target() == attach {
                        return;
                    }
                    if let Ok(other) = lasernodes.get(trigger.target()) {
                        other.connect(attach);
                        other.recalculate(&lasernodes);
                    }
                    if let Ok(this) = lasernodes.get(attach) {
                        this.connect(trigger.target());
                        this.recalculate(&lasernodes);
                    }
                },
            );
            sensor.observe(
                move |trigger: Trigger<OnCollisionEnd>, lasernodes: Query<&LaserNode>| {
                    if trigger.target() == attach {
                        return;
                    }
                    if let Ok(other) = lasernodes.get(trigger.target()) {
                        other.disconnect(attach);
                        other.recalculate(&lasernodes);
                    }
                    if let Ok(this) = lasernodes.get(attach) {
                        this.disconnect(trigger.target());
                        this.recalculate(&lasernodes);
                    }
                },
            );
        }
        PieceType::ScrapShip => {
            sensor.observe(
                move |trigger: Trigger<OnCollisionStart>, mut ships: Query<&mut ScrapShip>| {
                    if trigger.target() != attach {
                        if let Ok(mut ship) = ships.get_mut(attach) {
                            ship.enter(trigger.target());
                        }
                    }
                },
            );
            sensor.observe(
                move |trigger: Trigger<OnCollisionEnd>, mut ships: Query<&mut ScrapShip>| {
                    if trigger.target() != attach {
                        if let Ok(mut ship) = ships.get_mut(attach) {
                            ship.leave(trigger.target());
                        }
                    }
                },
            );
        }
        PieceType::BasicTurret
        | PieceType::BlastTurret
        | PieceType::LaserTurret
        | PieceType::SmartTurret => {
            sensor.observe(
                move |trigger: Trigger<OnCollisionStart>, mut turrets: Query<&mut Turret>| {
                    if trigger.target() != attach {
                        if let Ok(mut turret) = turrets.get_mut(attach) {
                            turret.enter(trigger.target());
                        }
                    }
                },
            );
            sensor.observe(
                move |trigger: Trigger<OnCollisionEnd>, mut turrets: Query<&mut Turret>| {
                    if trigger.target() != attach {
                        if let Ok(mut turret) = turrets.get_mut(attach) {
                            turret.leave(trigger.target());
                        }
                    }
                },
            );
        }
        _ => {}
    }
}

pub fn make_thing(
    mut commands: Commands,
    broadcast: ResMut<Sender>,
    mut things: EventReader<PlaceEvent>,
    territories: Query<(&GamePiece, &Transform, Option<&Fabber>, Option<&Territory>)>,
) {
    'evloop: for ev in things.read() {
        let mut transform = Transform::from_xyz(ev.x, ev.y, 0.0);
        transform.rotate_z(ev.a);
        let mut piece = commands.spawn((
            RigidBody::Dynamic,
            transform,
            ExternalForce::default(),
            ExternalImpulse::default(),
            CollisionEventsEnabled, // TODO: clear out unnecessary components from here so we don't have damping, impulse, etc on static bodies
        ));
        // fabber check
        let mut isfab = false;
        if ev.free {
            isfab = true;
        } else {
            for (territory_holder, position, fabber, territory) in territories.iter() {
                let d_x = position.translation.x - ev.x;
                let d_y = position.translation.y - ev.y;
                let dist = d_x * d_x + d_y * d_y;
                if let Some(fabber) = fabber {
                    if dist < fabber.radius * fabber.radius && fabber.is_available(ev.tp) {
                        if territory_holder.owner == ev.owner {
                            isfab = true;
                        }
                        if territory_holder.slot > 1 && ev.slot == territory_holder.slot {
                            isfab = true;
                        }
                    }
                }
                if let Some(territory) = territory {
                    if ev.tp == PieceType::Castle {
                        if dist.sqrt() < territory.radius + 600.0 {
                            if territory_holder.owner != ev.owner
                                && (territory_holder.slot == 1 || territory_holder.slot != ev.slot)
                            {
                                println!("too close!");
                                piece.despawn();
                                continue 'evloop;
                            }
                        }
                    }
                }
            }
        }
        if !isfab {
            piece.despawn();
            continue;
        }
        let mut health = 0.0;
        piece.insert(ev.tp.shape().to_collider());
        match ev.tp {
            PieceType::BasicFighter => {
                piece.insert((
                    Spaceshipoid::of(Ship::normal(), ev.x, ev.y),
                    Gun::mediocre(),
                ));
                health = 3.0;
            }
            PieceType::Castle => {
                let terr = Territory::castle();
                let fab = Fabber::castle();
                let _ = broadcast.send(ServerMessage::Territory {
                    id: piece.id().into(),
                    radius: terr.radius,
                });
                let _ = broadcast.send(ServerMessage::Fabber {
                    id: piece.id().into(),
                    radius: fab.radius,
                });
                piece.insert((terr, fab));
                health = 6.0;
            }
            PieceType::TieFighter => {
                piece.insert((
                    Spaceshipoid::of(Ship::normal(), ev.x, ev.y),
                    Gun::basic_repeater(2),
                ));
                health = 3.0;
            }
            PieceType::Sniper => {
                piece.insert((Spaceshipoid::of(Ship::fast(), ev.x, ev.y), Gun::sniper()));
                health = 3.0;
            }
            PieceType::DemolitionCruiser => {
                piece.insert((Spaceshipoid::of(Ship::slow(), ev.x, ev.y), Gun::bomber()));
                health = 3.0;
            }
            PieceType::Battleship => {
                piece.insert((
                    Spaceshipoid::of(Ship::slow(), ev.x, ev.y),
                    Gun::mediocre().extended_barrels(4, 40.0).offset(90.0),
                ));
                health = 12.0;
            }
            PieceType::Seed => {
                piece.insert(Seed::new());
                health = 1.0;
            }
            PieceType::Chest => {
                piece.insert(Chest {});
                health = 1.0;
            }
            PieceType::Farmhouse => {
                piece.insert(Farmhouse {});
                health = 2.0;
            }
            PieceType::BallisticMissile => {
                piece.insert(Spaceshipoid::of(Missile::ballistic(), ev.x, ev.y));
                health = 1.0;
            }
            PieceType::SeekingMissile => {
                piece.insert(Spaceshipoid::of(
                    Missile::cruise().with_intercept_burn(200.0),
                    ev.x,
                    ev.y,
                ));
                health = 1.0;
            }
            PieceType::HypersonicMissile => {
                piece.insert((
                    Spaceshipoid::of(Missile::hypersonic(), ev.x, ev.y),
                    CollisionExplosion {
                        explosion: ExplosionProperties {
                            damage: 1.0,
                            radius: 100.0,
                        },
                    },
                ));
                health = 1.0;
            }
            PieceType::TrackingMissile => {
                piece.insert((
                    Spaceshipoid::of(Missile::hypersonic().with_intercept_burn(200.0), ev.x, ev.y),
                    CollisionExplosion {
                        explosion: ExplosionProperties {
                            damage: 1.0,
                            radius: 100.0,
                        },
                    },
                ));
                health = 1.0;
            }
            PieceType::CruiseMissile => {
                piece.insert((
                    Spaceshipoid::of(Missile::cruise(), ev.x, ev.y),
                    CollisionExplosion {
                        explosion: ExplosionProperties {
                            damage: 4.0,
                            radius: 200.0,
                        },
                    },
                ));
            }
            PieceType::LaserNode => {
                piece.insert(LaserNode::new(2));
            }
            PieceType::LaserNodeLR => {
                piece.insert(LaserNode::new(2));
            }
            PieceType::ScrapShip => {
                piece.insert((
                    Spaceshipoid::of(Ship::normal(), ev.x, ev.y),
                    ScrapShip::new(),
                ));
            }
            PieceType::BasicTurret => {
                piece.insert((Turret::new(StandardTargeting), Gun::mediocre()));
            }
            PieceType::SmartTurret => {
                piece.insert((Turret::new(SmartTargeting), Gun::mediocre()));
            }
            _ => {}
        };
        piece.insert(GamePiece::new(ev.tp, ev.owner, ev.slot, health));
        let _ = broadcast.send(ServerMessage::ObjectCreate {
            x: ev.x,
            y: ev.y,
            a: ev.a,
            owner: ev.owner,
            id: piece.id().into(),
            tp: ev.tp,
        });
        let id = piece.id();
        if let Some(radius) = ev.tp.sensor() {
            let sensor = commands.spawn((
                FieldSensor::new(id),
                Collider::circle(radius),
                transform,
                Sensor,
                CollisionEventsEnabled,
            ));
            setup_sensor_observers(&ev.tp, id, sensor);
        }
    }
}
