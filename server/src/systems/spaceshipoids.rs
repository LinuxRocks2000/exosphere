// move spaceship-like pieces (missiles, ships)
// their defining characteristics are that they use PathFollower and the solve_spaceship functions in a stateless manner

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use common::pathfollower::PathNode;
use crate::solve_spaceship::*;
use crate::components::GamePiece;
use crate::resources::ClientMap;
use crate::components::Spaceshipoid;
use common::comms::ServerMessage;


pub fn move_spaceshipoids(mut shipoids : Query<(&mut ExternalImpulse, &Velocity, &Transform, &mut Spaceshipoid, &GamePiece, &Collider, Entity)>, targetables : Query<&Transform>, mut clients : ResMut<ClientMap>) {
    for (mut impulse, velocity, transform, mut spaceship, piece, collider, entity) in shipoids.iter_mut() {
        let mut goal = spaceship.kinematics.node_override();
        let mut is_override = true;
        let inv_mass = collider.raw.mass_properties(1.0).inv_mass;
        if let None = goal {
            is_override = false;
            goal = spaceship.pathfollower.get_next();
        }
        if let Some(goal) = goal {
            let cangle = transform.rotation.to_euler(EulerRot::ZYX).0;
            match match goal {
                PathNode::StraightTo(x, y) => {
                    let off = Vec2::new(x, y) - transform.translation.truncate();
                    spaceship.kinematics.to_position(off, cangle, velocity.linvel, velocity.angvel)
                },
                PathNode::Target(thing) => {
                    if let Ok(target) = targetables.get(thing.into()) {
                        let off = (target.translation - transform.translation).truncate();
                        spaceship.kinematics.to_position_tracking(off, cangle, velocity.linvel, velocity.angvel)
                    }
                    else {
                        KinematicResult::Done(Vec2::ZERO, 0.0)
                    }
                },
                PathNode::Rotation(ang, dur) => {
                    let off = loopify(cangle, ang);
                    spaceship.kinematics.to_angle(off, velocity.linvel, velocity.angvel)
                }
            } {
                KinematicResult::Thrust(imp, torque) => {
                    impulse.impulse = imp / inv_mass;
                    impulse.torque_impulse = torque / inv_mass;
                },
                KinematicResult::Done(imp, torque) => {
                    impulse.impulse = imp / inv_mass;
                    impulse.torque_impulse = torque / inv_mass;
                    if is_override {
                        spaceship.kinematics.override_complete();
                    }
                    else {
                        if let Ok(true) = spaceship.pathfollower.bump() {
                            if let Some(client) = clients.get_mut(&piece.owner) {
                                client.send(ServerMessage::StrategyCompletion {
                                    id : entity.into(),
                                    remaining: spaceship.pathfollower.len().unwrap() // unwrap should be safe here
                                });
                            }
                        }
                    }
                },
                KinematicResult::Noop => {}
            }
        }
    }
}