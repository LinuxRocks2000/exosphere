/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// turret control
use crate::components::*;
use crate::solve_spaceship::*;
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn turrets(
    mut turrets: Query<(
        &mut Turret,
        &GamePiece,
        &mut ExternalAngularImpulse,
        &Transform,
        &AngularVelocity,
    )>,
    pieces: Query<(&GamePiece, &Transform, &LinearVelocity, &Collider)>,
) {
    for (mut turret, turret_piece, mut torque, turret_pos, turret_angvel) in turrets.iter_mut() {
        for i in 0..turret.in_range.len() {
            if let Ok((piece, position, _, _)) = pieces.get(turret.in_range[i]) {
                if turret_piece.owner == piece.owner
                    || (turret_piece.slot > 1 && turret_piece.slot == piece.slot)
                {
                    //continue; // TODO: UNCOMMENT THIS!!!!
                }
                if turret.targeting_algorithm.will_attack(piece.tp) {
                    let ang = turret.targeting_algorithm.get_target_angle(
                        (position.translation - turret_pos.translation).truncate(),
                        piece.c_vel,
                    );
                    let c_ang = turret_pos.rotation.to_euler(EulerRot::ZYX).0;
                    torque.set_impulse(
                        turret
                            .targeting_algorithm
                            .swivel_kinematics(loopify(c_ang, ang), **turret_angvel),
                    );
                }
            } else {
                turret.in_range.swap_remove(i);
                break;
            }
        }
    }
}
