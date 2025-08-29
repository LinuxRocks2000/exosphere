/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// handles items that decay over time

use crate::components::*;
use crate::events::*;
use bevy::prelude::*;
use common::PlayerId;

pub fn ttl(
    mut expirees: Query<(Entity, &mut TimeToLive)>,
    mut kill_event: EventWriter<PieceDestroyedEvent>,
) {
    for (entity, mut ttl) in expirees.iter_mut() {
        if ttl.lifetime == 0 {
            kill_event.write(PieceDestroyedEvent {
                piece: entity,
                responsible: PlayerId::SYSTEM,
            });
        } else {
            ttl.lifetime -= 1;
        }
    }
}
