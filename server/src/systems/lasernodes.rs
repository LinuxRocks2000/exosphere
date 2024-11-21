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


// cast lasers between nodes
pub fn lasernodes(lasernodes : Query<(Entity, &LaserNode, &Transform)>, mut laser_cast : EventWriter<LaserCastEvent>) {
    for (entity, node, position) in lasernodes.iter() {
        for x in 0..node.allowable.min(node.slots.read().unwrap().len()) {
            if let Ok((_, other, otherposition)) = lasernodes.get(*node.slots.read().unwrap().get(x).unwrap()) {
                laser_cast.send(LaserCastEvent {
                    caster : entity,
                    from : position.translation.truncate(),
                    dir : (otherposition.translation - position.translation).truncate().normalize(),
                    max_dist : ((otherposition.translation - position.translation).length() - 12.0).max(0.0),
                    dmg : 1.0,
                    exclusive : None
                });
            }
        }
    }
}