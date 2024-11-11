/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::resources::*;
use crate::events::*;
use crate::components::*;
use common::types::*;
use crate::pathfollower::*;
use crate::comms::*;


pub fn make_thing(mut commands : Commands, broadcast : ResMut<Sender>, mut things : EventReader<PlaceEvent>, territories : Query<(&GamePiece, &Transform, Option<&Fabber>, Option<&Territory>)>) {
    'evloop: for ev in things.read() {
        let mut transform = Transform::from_xyz(ev.x, ev.y, 0.0);
        transform.rotate_z(ev.a);
        let mut piece = commands.spawn((RigidBody::Dynamic, Velocity::zero(), TransformBundle::from(transform), ExternalForce::default(), ExternalImpulse::default(), Damping {
            linear_damping : 0.0,// todo: clear out unnecessary components (move them to the match statement so we don't have, say, ExternalImpulse on a static body)
            angular_damping : 0.0
        }, ActiveEvents::COLLISION_EVENTS));
        // fabber check
        let mut isfab = false;
        if ev.free {
            isfab = true;
        }
        else {
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
                            if territory_holder.owner != ev.owner && (territory_holder.slot == 1 || territory_holder.slot != ev.slot) {
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
        match ev.tp {
            PieceType::BasicFighter => {
                piece.insert((ev.tp.shape().to_collider(), PathFollower::start(ev.x, ev.y), Ship::normal(), Gun::mediocre()));
                health = 3.0;
            },
            PieceType::Castle => {
                let terr = Territory::castle();
                let fab = Fabber::castle();
                let _ = broadcast.send(ServerMessage::Territory(piece.id().index(), terr.radius));
                let _ = broadcast.send(ServerMessage::Fabber(piece.id().index(), fab.radius));
                piece.insert((ev.tp.shape().to_collider(), terr, fab));
                health = 6.0;
            },
            PieceType::TieFighter => {
                piece.insert((Collider::cuboid(20.0, 25.0), PathFollower::start(ev.x, ev.y), Ship::normal(), Gun::basic_repeater(2)));
                health = 3.0;
            },
            PieceType::Sniper => {
                piece.insert((Collider::cuboid(30.0, 15.0), PathFollower::start(ev.x, ev.y), Ship::fast(), Gun::sniper()));
                health = 3.0;
            },
            PieceType::DemolitionCruiser => {
                piece.insert((Collider::cuboid(20.0, 20.0), PathFollower::start(ev.x, ev.y), Ship::slow(), Gun::bomber()));
                health = 3.0;
            },
            PieceType::Battleship => {
                piece.insert((Collider::cuboid(75.0, 100.0), PathFollower::start(ev.x, ev.y), Ship::slow(), Gun::mediocre().extended_barrels(4, 40.0).offset(90.0)));
                health = 12.0;
            },
            PieceType::Seed => {
                piece.insert((Collider::cuboid(3.5, 3.5), Seed::new()));
                health = 1.0;
            },
            PieceType::Chest => {
                piece.insert((Collider::cuboid(10.0, 10.0), Chest{}));
                health = 1.0;
            },
            PieceType::Farmhouse => {
                piece.insert((Collider::cuboid(25.0, 25.0), Farmhouse {}));
                health = 2.0;
            },
            PieceType::BallisticMissile => {
                piece.insert((Collider::cuboid(17.5, 10.0), Missile::ballistic(), PathFollower::start(ev.x, ev.y)));
                health = 1.0;
            },
            PieceType::SeekingMissile => {
                piece.insert((Collider::cuboid(17.5, 10.0), Missile::cruise().with_intercept_burn(200.0), PathFollower::start(ev.x, ev.y)));
                health = 1.0;
            },
            PieceType::HypersonicMissile => {
                piece.insert((Collider::cuboid(17.5, 5.0), Missile::hypersonic(), PathFollower::start(ev.x, ev.y), CollisionExplosion {
                    explosion : ExplosionProperties {
                        damage : 1.0,
                        radius : 100.0
                    }
                }));
                health = 1.0;
            },
            PieceType::TrackingMissile => {
                piece.insert((Collider::cuboid(17.5, 8.5), Missile::hypersonic().with_intercept_burn(200.0), PathFollower::start(ev.x, ev.y).with_tracking(), CollisionExplosion {
                    explosion : ExplosionProperties {
                        damage : 1.0,
                        radius : 100.0
                    }
                }));
                health = 1.0;
            },
            PieceType::CruiseMissile => {
                piece.insert((Collider::cuboid(17.5, 5.0), Missile::cruise(), PathFollower::start(ev.x, ev.y), CollisionExplosion {
                    explosion : ExplosionProperties {
                        damage : 4.0,
                        radius : 200.0
                    }
                }));
            },
            PieceType::Bullet => {}, // not implemented: bullets must be created by the discharge_barrel function
            PieceType::SmallBomb => {}, // same
            PieceType::FleetDefenseShip => {} // TODO: fleet defense ships
        };
        piece.insert(GamePiece::new(ev.tp, ev.owner, ev.slot, health));
        let _ = broadcast.send(ServerMessage::ObjectCreate(ev.x, ev.y, ev.a, ev.owner, piece.id().index(), ev.tp as u16));
        let id = piece.id();
        if let PieceType::Farmhouse = ev.tp {
            commands.spawn((FieldSensor::farmhouse(id), Collider::ball(100.0), TransformBundle::from(transform), Sensor, ActiveEvents::COLLISION_EVENTS));
        }
        if let PieceType::SeekingMissile = ev.tp {
            commands.spawn((FieldSensor::farmhouse(id), Collider::ball(300.0), TransformBundle::from(transform), Sensor, ActiveEvents::COLLISION_EVENTS));
        }
    }
}