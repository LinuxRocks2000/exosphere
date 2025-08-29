/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// fires bullets from items with guns

use crate::components::*;
use crate::resources::*;
use crate::PieceType;
use avian2d::prelude::*;
use bevy::prelude::*;
use common::comms::ServerMessage;
use common::PlayerId;
use rand::Rng;

#[derive(Copy, Clone)]
pub enum Bullets {
    MinorBullet(u16),               // simple bullet with range
    Bomb(ExplosionProperties, u16), // properties of the explosion we're boutta detonate, range of the bullet
}

fn discharge_barrel(
    commands: &mut Commands,
    owner: PlayerId,
    barrel: u16,
    gun: &Gun,
    position: &Transform,
    velocity: &LinearVelocity,
    broadcast: &ResMut<Sender>,
) {
    let ang = position.rotation.to_euler(EulerRot::ZYX).0;
    let vel = LinearVelocity(**velocity + glam::f32::Vec2::from_angle(ang) * 450.0);
    let mut transform = position.clone();
    transform.translation += (Vec2::from_angle(ang) * gun.center_offset).extend(0.0);
    transform.translation += (Vec2::from_angle(ang).perp()
        * gun.barrel_spacing
        * (barrel as f32 - gun.barrels as f32 / 2.0 + 0.5))
        .extend(0.0);
    match gun.bullets {
        Bullets::MinorBullet(range) => {
            let piece = commands.spawn((
                GamePiece::new(PieceType::Bullet, owner, 0, 0.5),
                RigidBody::Dynamic,
                PieceType::shape(&PieceType::Bullet).to_collider(),
                vel,
                transform,
                TimeToLive { lifetime: range },
                Bullet { tp: gun.bullets },
                CollisionEventsEnabled,
                PresolveVelocity(Vec2::new(0.0, 0.0)),
            ));
            let _ = broadcast.send(ServerMessage::ObjectCreate {
                x: transform.translation.x,
                y: transform.translation.y,
                a: ang,
                owner: PlayerId::SYSTEM,
                id: piece.id().into(),
                tp: PieceType::Bullet,
            });
        }
        Bullets::Bomb(_, range) => {
            let piece = commands.spawn((
                GamePiece::new(PieceType::SmallBomb, owner, 0, 0.5),
                RigidBody::Dynamic,
                PieceType::shape(&PieceType::SmallBomb).to_collider(),
                vel,
                transform,
                TimeToLive { lifetime: range },
                Bullet { tp: gun.bullets },
                CollisionEventsEnabled,
                PresolveVelocity(Vec2::new(0.0, 0.0)),
            ));
            let _ = broadcast.send(ServerMessage::ObjectCreate {
                x: transform.translation.x,
                y: transform.translation.y,
                a: ang,
                owner: PlayerId::SYSTEM,
                id: piece.id().into(),
                tp: PieceType::SmallBomb,
            });
        }
    }
}

pub fn shoot(
    mut commands: Commands,
    mut pieces: Query<(&Transform, &LinearVelocity, &GamePiece, &mut Gun)>,
    broadcast: ResMut<Sender>,
) {
    for (position, velocity, piece, mut gun) in pieces.iter_mut() {
        if gun.enabled {
            if gun.tick == 0 {
                gun.r_point += 1;
                if gun.r_point >= gun.repeats {
                    gun.tick = gun.cd;
                    gun.r_point = 0;
                } else {
                    gun.tick = gun.repeat_cd;
                }
                if gun.scatter_barrels {
                    discharge_barrel(
                        &mut commands,
                        piece.owner,
                        rand::thread_rng().gen_range(0..gun.barrels),
                        &gun,
                        position,
                        velocity,
                        &broadcast,
                    );
                } else {
                    for barrel in 0..gun.barrels {
                        discharge_barrel(
                            &mut commands,
                            piece.owner,
                            barrel,
                            &gun,
                            position,
                            velocity,
                            &broadcast,
                        );
                    }
                }
            }
            gun.tick -= 1;
        }
    }
}
