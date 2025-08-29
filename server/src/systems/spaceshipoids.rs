/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// move spaceship-like pieces (missiles, ships)
// their defining characteristics are that they use PathFollower and the solve_spaceship functions in a stateless manner

use crate::components::*;
use crate::resources::ClientMap;
use crate::solve_spaceship::*;
use avian2d::prelude::*;
use bevy::prelude::*;
use common::comms::ServerMessage;
use common::pathfollower::PathNode;

pub fn move_spaceshipoids(
    mut shipoids: Query<(
        &mut ExternalImpulse,
        &mut ExternalTorque,
        &LinearVelocity,
        &AngularVelocity,
        &Transform,
        &mut Spaceshipoid,
        &GamePiece,
        &Collider,
        Entity,
    )>,
    targetables: Query<&Transform>,
    mut clients: ResMut<ClientMap>,
    chan: Query<&ClientChannel>,
) {
    for (
        mut impulse,
        mut torque,
        linvel,
        angvel,
        transform,
        mut spaceship,
        piece,
        collider,
        entity,
    ) in shipoids.iter_mut()
    {
        let mut goal = spaceship.kinematics.node_override();
        let mut is_override = true;
        let mass = collider.mass_properties(1.0).mass;
        if let None = goal {
            is_override = false;
            goal = spaceship.pathfollower.get_next();
        }
        if let Some(goal) = goal {
            let cangle = transform.rotation.to_euler(EulerRot::ZYX).0;
            match match goal {
                PathNode::StraightTo(x, y) => {
                    let off = Vec2::new(x, y) - transform.translation.truncate();
                    spaceship
                        .kinematics
                        .to_position(off, cangle, **linvel, **angvel)
                }
                PathNode::Target(thing) => {
                    if let Ok(target) = targetables.get(thing.into()) {
                        let off = (target.translation - transform.translation).truncate();
                        spaceship
                            .kinematics
                            .to_position_tracking(off, cangle, **linvel, **angvel)
                    } else {
                        KinematicResult::Done(Vec2::ZERO, 0.0)
                    }
                }
                PathNode::Rotation(ang, _) => {
                    let off = loopify(cangle, ang);
                    spaceship.kinematics.to_angle(off, **linvel, **angvel)
                }
            } {
                KinematicResult::Thrust(imp, t) => {
                    impulse.set_impulse(imp * mass);
                    torque.set_torque(t * mass);
                }
                KinematicResult::Done(imp, t) => {
                    impulse.set_impulse(imp * mass);
                    torque.set_torque(t * mass);
                    if is_override {
                        spaceship.kinematics.override_complete();
                    } else {
                        if let Ok(true) = spaceship.pathfollower.bump() {
                            if let Some(client) = clients.get_mut(&piece.owner) {
                                chan.get(*client).unwrap().send(
                                    ServerMessage::StrategyCompletion {
                                        id: entity.into(),
                                        remaining: spaceship.pathfollower.len().unwrap(), // unwrap should be safe here
                                    },
                                );
                            }
                        }
                    }
                }
                KinematicResult::Noop => {}
            }
        }
        if impulse.impulse().length() > 300000.0 {
            impulse.set_impulse(Vec2::ZERO);
        }
    }
}
