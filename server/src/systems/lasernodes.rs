/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>. 
*/

// lasernode handler system
use rayon::prelude::*;
use crate::components::*;
use crate::events::*;
use bevy::prelude::*;
use common::types::*;

// cast lasers between nodes
pub fn lasernodes(lasernodes : Query<(Entity, &GamePiece, &LaserNode, &Transform)>, mut laser_cast : EventWriter<LaserCastEvent>) {
    lasernodes.par_iter().for_each(|(entity, _, node, position)| {
        let slots = node.slots.read().unwrap();
        let slot_count = slots.len().min(node.allowable);

        for i in 0..slot_count {
            if let Ok((_, other_piece, _, other_position)) = lasernodes.get(slots[i]) {
                let direction = (other_position.translation - position.translation).truncate().normalize();
                let distance = (other_position.translation - position.translation).length();
                let max_dist = distance - match other_piece.tp {
                    PieceType::LaserNode => 12.0,
                    PieceType::LaserNodeLR => 23.0,
                    _ => 0.0,
                }.max(0.0);

                laser_cast.send(LaserCastEvent {
                    caster: entity,
                    from: position.translation.truncate(),
                    dir: direction,
                    max_dist,
                    dmg: 1.0,
                    exclusive: None,
                });
            }
        }
    });
}
