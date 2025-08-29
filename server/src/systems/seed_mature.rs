/*
    Copyright 2024 Tyler Clarke.

    This file is part of Exosphere.

    Exosphere is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

    Exosphere is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along with Exosphere. If not, see <https://www.gnu.org/licenses/>.
*/

// handles seed maturation

use crate::components::*;
use crate::events::*;
use crate::placer::Placer;
use bevy::prelude::*;
use common::PlayerId;

pub fn seed_mature(
    mut seeds: Query<(Entity, &Transform, &mut Seed)>,
    place: EventWriter<PlaceEvent>,
    mut destroy: EventWriter<PieceDestroyedEvent>,
) {
    let mut place = Placer(place);
    for (entity, transform, mut seed) in seeds.iter_mut() {
        if seed.growing {
            seed.time_to_grow -= 1;
        }
        if seed.time_to_grow == 0 {
            destroy.write(PieceDestroyedEvent {
                piece: entity,
                responsible: PlayerId::SYSTEM,
            });
            place.chest_free(transform.translation.x, transform.translation.y);
        }
    }
}
