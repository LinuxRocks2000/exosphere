/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// produces explosions

use crate::events::*;
use crate::resources::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use common::comms::ServerMessage;
use common::PlayerId;

pub fn boom(
    mut commands: Commands,
    mut explosions: EventReader<ExplosionEvent>,
    sender: ResMut<Sender>,
) {
    // manage explosions
    // explosions are really just sensored colliders with an explosionproperties
    for explosion in explosions.read() {
        let _ = sender.send(ServerMessage::Explosion {
            x: explosion.x,
            y: explosion.y,
            radius: explosion.props.radius,
            damage: explosion.props.damage,
        });
        let damage = explosion.props.damage;
        commands
            .spawn((
                RigidBody::Dynamic,
                explosion.props,
                Collider::circle(explosion.props.radius),
                Transform::from_xyz(explosion.x, explosion.y, 0.0),
                CollisionEventsEnabled,
                Sensor,
            ))
            .observe(
                move |trigger: Trigger<OnCollisionStart>, mut hurt: EventWriter<PieceHarmEvent>| {
                    hurt.write(PieceHarmEvent {
                        piece: trigger.collider,
                        harm_amount: damage,
                        responsible: PlayerId::SYSTEM, // TODO: use the actual responsible id
                    });
                },
            );
    }
}
