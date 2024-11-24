/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// turret control
use crate::components::*;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::solve_spaceship::*;
use crate::events::*;


pub fn turrets(mut turrets : Query<(Entity, &mut Turret, &GamePiece, &mut ExternalImpulse, &Transform, &Velocity)>, pieces : Query<(&GamePiece, &Transform, &Velocity, &Collider)>) {
    for (mut turret, turret_piece, mut impulse, turret_pos, turret_velocity) in turrets.iter_mut() {
        for i in 0..turret.in_range.len() {
            if let Ok((piece, position, velocity, collider)) = pieces.get(turret.in_range[i]) {
                if turret_piece.owner == piece.owner || (turret_piece.slot > 1 && turret_piece.slot == piece.slot) {
                    continue;
                }
                if turret.targeting_algorithm.will_attack(piece.tp) {
                    let ang = turret.targeting_algorithm.get_target_angle((position.translation - turret_pos.translation).truncate(), piece.c_vel);
                    let c_ang = turret_pos.rotation.to_euler(EulerRot::ZYX).0;
                    impulse.torque_impulse = turret.targeting_algorithm.swivel_kinematics(loopify(c_ang, ang), turret_velocity.angvel);
                }
            }
            else {
                turret.in_range.swap_remove(i);
                break;
            }
        }
    }
}