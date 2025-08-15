/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// lasernode handler system

use crate::components::*;
use crate::events::*;
use bevy::prelude::*;
use common::types::*;

// cast lasers between nodes
pub fn lasernodes(
    lasernodes: Query<(Entity, &GamePiece, &LaserNode, &Transform)>,
    mut laser_cast: EventWriter<LaserCastEvent>,
) {
    for (entity, _, node, position) in lasernodes.iter() {
        for x in 0..node.slots.read().unwrap().len() {
            if let Ok((_, opiece, _, otherposition)) =
                lasernodes.get(*node.slots.read().unwrap().get(x).unwrap())
            {
                laser_cast.write(LaserCastEvent {
                    caster: entity,
                    from: position.translation.truncate(),
                    dir: (otherposition.translation - position.translation)
                        .truncate()
                        .normalize(),
                    max_dist: ((otherposition.translation - position.translation).length()
                        - match opiece.tp {
                            PieceType::LaserNode => 12.0,
                            PieceType::LaserNodeLR => 23.0,
                            _ => 0.0,
                        })
                    .max(0.0),
                    dmg: 1.0,
                    exclusive: None,
                });
            }
        }
    }
}
